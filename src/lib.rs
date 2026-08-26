//! Relaxed key generation

#![no_std]
#![forbid(unused_crate_dependencies, missing_docs)]

extern crate alloc;
#[cfg(test)]
extern crate std;

mod _unused_deps {
    // We don't use it directly, but we need to enable `serde` feature to make `sha2::Output`
    // (=`GenericArray`) implement `Serialize` and `Deserialze`.
    // `generic_array` on 0.14 is needed by `sha2/digest` on 0.10 (needed by `generic-ec`), so we
    // can't upgrade `generic_array` to 1 yet.
    #[allow(deprecated)]
    use generic_array as _;
}

use alloc::{string::String, vec::Vec};
use core::iter::once;
use generic_ec::{Curve, Point, Scalar, SecretScalar};
use generic_ec_zkp::polynomial::Polynomial;
use round_based::{
    PartyIndex, ProtocolMsg,
    mpc::{Mpc, MpcExecution},
};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, digest::Output};
use udigest::{Bytes, Digestable, hash};

/// Protocol message
#[derive(ProtocolMsg, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")] // Clears `E: Serialize + Deserializae` from derive
pub enum Msg<E: Curve> {
    /// Round 1 broadcast: commitment to the vector of curve points
    CommitPoints(CommitPointsMsg),
    /// Round 1 pairwise: commitment to a receiver's subshare
    CommitSubshare(CommitSubshareMsg),
    /// Round 2: broadcast: echo commitments and decommit the vector of curve points
    EchoDigestAndDecommitPoints(EchoDigestAndDecommitPointsMsg<E>),
    /// Round 2 pairwise: decommit a receiver's subshare
    DecommitSubshare(DecommitSubshareMsg<E>),
    /// Round 3 verify
    Verify(VerifyMsg),
}

/// Round 1 broadcast: points commitment
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Digestable)]
pub struct CommitPointsMsg {
    /// Party commitment
    #[udigest(as_bytes)]
    commitment: Output<Sha256>,
}

/// Round 1 pairwise: subshare commitment
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Digestable)]
pub struct CommitSubshareMsg {
    /// Party commitment
    #[udigest(as_bytes)]
    commitment: Output<Sha256>,
}

/// Round 2 broadcast: echo agreement and opening of the curve-points commitment.
///
/// Carries the values needed to recompute the digest for committed points.
/// Echos digest for points commitments
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Digestable)]
#[serde(bound = "")] // Clears the requirement on `E: Serialize + Deserialize`
pub struct EchoDigestAndDecommitPointsMsg<E: Curve> {
    /// Party commitments as digest
    #[udigest(as_bytes)]
    digest: Output<Sha256>,
    /// The committed vector of curve points
    points: Vec<Point<E>>,
    /// Commitment nonce
    nonce: [u8; NONCE_BYTES],
}

/// Round 2 pairwise: opening of a subshare commitment.
///
/// Carries the values needed to recompute the digest for committed subshares.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct DecommitSubshareMsg<E: Curve> {
    /// The committed subshare
    subshare: Scalar<E>,
    /// Commitment nonce
    nonce: [u8; NONCE_BYTES],
}

/// Round 3 broadcast verificiation result
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyMsg {
    ok: bool,
}

const LAMBDA_BITS: usize = 128;
const NONCE_BYTES: usize = 2 * LAMBDA_BITS / 8;

#[derive(Digestable)]
#[udigest(tag = "dkls23.keygen.committed_points")]
#[udigest(bound = "")]
struct CommittedPoints<'sid, 'points, E: Curve> {
    sid: &'sid [u8],
    committer: PartyIndex,
    points: &'points [Point<E>],
    #[udigest(as_bytes)]
    nonce: [u8; NONCE_BYTES],
}

#[derive(Digestable)]
#[udigest(tag = "dkls23.keygen.committed_subshare")]
#[udigest(bound = "")]
struct CommittedSubshare<'sid, E: Curve> {
    sid: &'sid [u8],
    committer: PartyIndex,
    receiver: PartyIndex,
    subshare: Scalar<E>,
    #[udigest(as_bytes)]
    nonce: [u8; NONCE_BYTES],
}

#[derive(Digestable)]
#[udigest(tag = "dkls23.keygen.echo_commitments")]
struct EchoCommitments<'a> {
    #[udigest(as = &[Bytes])]
    commitments: &'a [Output<Sha256>],
}

enum Verification<E: Curve> {
    Abort(String),
    Success(KeyShare<E>),
}

/// Key share output from the protocol
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct KeyShare<E: Curve> {
    /// Public key P(0)
    share_point_0: Point<E>,
    /// Private key share s_i := p(i + 1) for 0-based party index
    share_i: Scalar<E>,
}

/// Carries out the key generation protocol
pub async fn relaxed_key_generation<R, M, E>(
    mut rng: R,
    i: PartyIndex, // Party 0, 1, ..., n - 1
    t: PartyIndex, // `1 <= t <= n`
    n: PartyIndex, // `2 <= n`
    mut mpc: M,
    sid: &[u8],
) -> Result<KeyShare<E>, ErrorM<M>>
where
    M: Mpc<Msg = Msg<E>>,
    R: rand_core::RngCore,
    E: generic_ec::Curve,
{
    let (i, t, n) = check_arguments(i, t, n)?;
    let me = i + 1; // 1-based party label for 0-based party index, for indexing curve points

    // Define rounds
    let round1_commit_points =
        mpc.add_round(round_based::round::broadcast::<CommitPointsMsg>(i, n));
    let round1_commit_subshares = mpc.add_round(round_based::round::p2p::<CommitSubshareMsg>(i, n));
    let round2_echo_digest_and_decommit_points = mpc.add_round(round_based::round::broadcast::<
        EchoDigestAndDecommitPointsMsg<E>,
    >(i, n));
    let round2_decommit_subshares =
        mpc.add_round(round_based::round::p2p::<DecommitSubshareMsg<E>>(i, n));
    let round3_verify = mpc.add_round(round_based::round::broadcast::<VerifyMsg>(i, n));

    let mut mpc = mpc.finish_setup();

    // --- The Protocol ---

    // 1. Generate local randomness
    let p_i = Polynomial::<SecretScalar<E>>::sample(&mut rng, usize::from(t) - 1);
    // n subshares p_i(j) for j = 1, 2, ..., n for party 0, 1, ..., n - 1
    let subshares: Vec<Scalar<E>> = (1..=n)
        .map(|j| p_i.value::<_, Scalar<E>>(&Scalar::<E>::from(j)))
        .collect();
    // t points P_i(j) for j = 0, 1, ..., t - 1, where P_i(0) is for the secret, and rest for party
    // 0, ..., t - 2.
    let subshare_curve_points: Vec<Point<E>> =
        once(p_i.value::<_, Scalar<E>>(&Scalar::<E>::from(0)))
            .chain((1..t).map(|j| subshares[usize::from(j) - 1]))
            .map(|s| s * Point::<E>::generator())
            .collect();

    let mut nonce = [0u8; NONCE_BYTES];
    rng.fill_bytes(&mut nonce);
    let committed_points = CommittedPoints {
        sid,
        committer: i,
        points: &subshare_curve_points,
        nonce,
    };
    let points_commitment = hash::<Sha256>(&committed_points);

    // Broadcast commitment to the vector of curve points.
    mpc.send_to_all(Msg::CommitPoints(CommitPointsMsg {
        commitment: points_commitment,
    }))
    .await
    .map_err(Error::Round1Send)?;

    // Pairwise commitment to each receiver's subshare.
    let committed_subshares: Vec<CommittedSubshare<E>> = iter_peers(i, n)
        .map(|j| CommittedSubshare {
            sid,
            committer: i,
            receiver: j,
            subshare: subshares[j as usize],
            nonce: {
                let mut nonce = [0u8; NONCE_BYTES];
                rng.fill_bytes(&mut nonce);
                nonce
            },
        })
        .collect();

    for committed_subshare in &committed_subshares {
        mpc.send_p2p(
            committed_subshare.receiver,
            Msg::CommitSubshare(CommitSubshareMsg {
                commitment: hash::<Sha256>(committed_subshare),
            }),
        )
        .await
        .map_err(Error::Round1Send)?;
    }

    let received_points_commitments = mpc
        .complete(round1_commit_points)
        .await
        .map_err(Error::Round1Receive)?;
    let received_subshare_commitments = mpc
        .complete(round1_commit_subshares)
        .await
        .map_err(Error::Round1Receive)?;

    // 2. decomit
    // Echo for broadcast points commitments
    let other_points_commitments: Vec<Output<Sha256>> = received_points_commitments
        .iter()
        .map(|c| c.commitment)
        .collect();
    // Rebuild the full n-length commitment vector in absolute party order for the echo digest.
    let all_points_commitments = [
        &other_points_commitments[0..i as usize],
        &[points_commitment],
        &other_points_commitments[i as usize..],
    ]
    .concat();

    let echo_digest = hash::<Sha256>(&EchoCommitments {
        commitments: &all_points_commitments,
    });

    // Echo agreement and open for points and subshare commitments
    mpc.send_to_all(Msg::EchoDigestAndDecommitPoints(
        EchoDigestAndDecommitPointsMsg {
            digest: echo_digest,
            points: subshare_curve_points.clone(),
            nonce,
        },
    ))
    .await
    .map_err(Error::Round2Send)?;

    for committed_subshare in &committed_subshares {
        mpc.send_p2p(
            committed_subshare.receiver,
            Msg::DecommitSubshare(DecommitSubshareMsg {
                subshare: committed_subshare.subshare,
                nonce: committed_subshare.nonce,
            }),
        )
        .await
        .map_err(Error::Round2Send)?;
    }

    let received_echo_digests_and_decommit_points = mpc
        .complete(round2_echo_digest_and_decommit_points)
        .await
        .map_err(Error::Round2Receive)?;
    let received_decommit_subshares = mpc
        .complete(round2_decommit_subshares)
        .await
        .map_err(Error::Round2Receive)?;

    // 3. Verify
    // Echo agreement
    let verification = if received_echo_digests_and_decommit_points
        .iter()
        .any(|m| m.digest != echo_digest)
    {
        // Send abort
        let verify = VerifyMsg { ok: false };
        mpc.send_to_all(Msg::Verify(verify))
            .await
            .map_err(Error::Round3Send)?;
        // Go to output
        Verification::Abort("echo agreement".into())
    } else if received_echo_digests_and_decommit_points
        .iter_indexed()
        .zip(received_points_commitments.iter_indexed())
        .any(|((j, _, d), (_, _, c))| {
            let committed_points = CommittedPoints {
                sid,
                committer: j,
                points: &d.points,
                nonce: d.nonce,
            };
            c.commitment != hash::<Sha256>(&committed_points)
        })
    {
        // Send abort
        let verify = VerifyMsg { ok: false };
        mpc.send_to_all(Msg::Verify(verify))
            .await
            .map_err(Error::Round3Send)?;
        // Go to output
        Verification::Abort("points commitment".into())
    } else if received_decommit_subshares
        .iter_indexed()
        .zip(received_subshare_commitments.iter_indexed())
        .any(|((j, _, d), (_, _, c))| {
            let committed_subshare = CommittedSubshare {
                sid,
                committer: j,
                receiver: i,
                subshare: d.subshare,
                nonce: d.nonce,
            };
            c.commitment != hash::<Sha256>(&committed_subshare)
        })
    {
        // Send abort
        let verify = VerifyMsg { ok: false };
        mpc.send_to_all(Msg::Verify(verify))
            .await
            .map_err(Error::Round3Send)?;
        // Go to output
        Verification::Abort("subshare commitment".into())
    } else {
        // Verify expected share curve point
        let share_points: Vec<Point<E>> = (0..t)
            .map(|k| {
                received_echo_digests_and_decommit_points
                    .iter()
                    .map(|p| p.points[k as usize])
                    .sum::<Point<E>>()
                    + subshare_curve_points[k as usize]
            })
            .collect();
        let share: Scalar<E> = received_decommit_subshares
            .iter()
            .map(|s| s.subshare)
            .sum::<Scalar<E>>()
            + subshares[i as usize];
        let share_curve_point = share * Point::<E>::generator();

        // Expected share curve point
        let expected_share_curve_point: Point<E> = if me < t {
            share_points[me as usize]
        } else {
            // Build from lagrange t curve points. Take care to use 1-based party label
            let label_set: Vec<Scalar<E>> =
                (1..=t - 1).chain([me]).map(Scalar::<E>::from).collect();
            let l_me_inv = l0(&label_set, Scalar::<E>::from(me))?
                .invert()
                .ok_or(InternalErr::ArithmeticError("l_me invert".into()))?;
            let mut acc = l0(&label_set, Scalar::<E>::from(1))? * share_points[1];
            for k in 2..=t - 1 {
                acc += l0(&label_set, Scalar::<E>::from(k))? * share_points[k as usize];
            }
            l_me_inv * (share_points[0] - acc)
        };

        if share_curve_point != expected_share_curve_point {
            // Send abort
            let verify = VerifyMsg { ok: false };
            mpc.send_to_all(Msg::Verify(verify))
                .await
                .map_err(Error::Round3Send)?;
            // Go to output
            Verification::Abort("expected share point".into())
        } else {
            // Send ok
            let verify = VerifyMsg { ok: true };
            mpc.send_to_all(Msg::Verify(verify))
                .await
                .map_err(Error::Round3Send)?;
            // Go to output
            Verification::Success(KeyShare {
                share_point_0: share_points[0],
                share_i: share,
            })
        }
    };

    let received_verifications = mpc
        .complete(round3_verify)
        .await
        .map_err(Error::Round3Receive)?;

    // Output
    if received_verifications.iter().any(|v| !v.ok) {
        Err(Error::Abort {
            msg: "received abort".into(),
        })
    } else {
        match verification {
            Verification::Abort(msg) => Err(Error::Abort { msg }),
            Verification::Success(key_share) => Ok(key_share),
        }
    }
}

fn check_arguments(
    i: PartyIndex,
    t: PartyIndex,
    n: PartyIndex,
) -> Result<(PartyIndex, PartyIndex, PartyIndex), InternalErr> {
    if 2 <= n && (1 <= t && t <= n) && (i < n) {
        Ok((i, t, n))
    } else {
        Err(InternalErr::InvalidArgument("i={i}, n={n}, t={t}".into()))
    }
}

fn l0<E: Curve>(s: &[Scalar<E>], k: Scalar<E>) -> Result<Scalar<E>, InternalErr> {
    lagrange(s, k, Scalar::<E>::zero())
}

fn lagrange<E: Curve>(
    s: &[Scalar<E>],
    k: Scalar<E>,
    x: Scalar<E>,
) -> Result<Scalar<E>, InternalErr> {
    let mut prod = Scalar::<E>::one();
    for &l in s {
        if l != k {
            prod *= (x - l)
                * (k - l)
                    .invert()
                    .ok_or(InternalErr::ArithmeticError("lagrange invert".into()))?;
        }
    }
    Ok(prod)
}

/// Iterate peers of i-th party. Stolen from [cggmp21](https://github.com/LFDT-Lockness/cggmp21)
pub fn iter_peers(i: u16, n: u16) -> impl Iterator<Item = u16> {
    (0..n).filter(move |x| *x != i)
}

/// Internal error
#[derive(Debug, thiserror::Error)]
pub enum InternalErr {
    /// Argument error
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    /// Arithmetic error
    #[error("arithmetic error at: {0}")]
    ArithmeticError(String),
}

/// Protocol error
#[derive(Debug, thiserror::Error)]
pub enum Error<RecvErr, SendErr> {
    /// Couldn't send a message in the first round
    #[error("send a message at round 1")]
    Round1Send(#[source] SendErr),
    /// Couldn't receive a message in the first round
    #[error("receive messages at round 1")]
    Round1Receive(#[source] RecvErr),
    /// Couldn't send a message in the second round
    #[error("send a message at round 2")]
    Round2Send(#[source] SendErr),
    /// Couldn't receive a message in the second round
    #[error("receive messages at round 2")]
    Round2Receive(#[source] RecvErr),
    /// Couldn't send a message in the third round
    #[error("send a message at round 3")]
    Round3Send(#[source] SendErr),
    /// Couldn't receive a message in the third round
    #[error("receive messages at round 3")]
    Round3Receive(#[source] RecvErr),

    /// Internal error
    #[error("Internal error")]
    InternalError(#[source] InternalErr),

    /// The protocol was aborted because a check failed (e.g. a decommitment
    /// didn't match its commitment, or the echo agreement check failed).
    #[error("protocol aborted: {msg}")]
    Abort {
        /// Abort message
        msg: String,
    },
}

/// Error type deduced from `M: Mpc`
pub type ErrorM<M> = Error<
    round_based::mpc::CompleteRoundErr<M, round_based::round::RoundInputError>,
    <M as Mpc>::SendErr,
>;

impl<RecvErr, SendErr> From<InternalErr> for Error<RecvErr, SendErr> {
    fn from(e: InternalErr) -> Self {
        Error::InternalError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyShare, relaxed_key_generation};
    use alloc::vec::Vec;
    use generic_ec::{Curve, Point, Scalar, curves::Secp256k1};
    use generic_ec_zkp::polynomial::lagrange_coefficient;
    use rand::seq::SliceRandom;

    const SID: &[u8] = b"test-session";

    /// Covers
    /// - the direct branch (party labels `<= t-1`),
    /// - the Lagrange branch (labels `>= t`),
    /// - the full-threshold `t = n`
    const CASES: &[(u16, u16)] = &[(2, 2), (2, 3), (3, 3), (3, 5), (5, 5)];

    /// Interpolates the secret-sharing polynomial through `polynomial_points` evaluated at `x`,
    /// where x is not an coordinate of one of the points.
    /// The `polynomial_points` are (1-based label, share) pairs.
    fn interpolate_at<E: Curve>(polynomial_points: &[(u16, Scalar<E>)], x: Scalar<E>) -> Scalar<E> {
        let xs: Vec<Scalar<E>> = polynomial_points
            .iter()
            .map(|(label, _)| Scalar::from(*label))
            .collect();
        polynomial_points
            .iter()
            .enumerate()
            .map(|(j, (_, share))| {
                lagrange_coefficient(x, j, &xs)
                    .expect("x is not part of one of the polynomial points")
                    * *share
            })
            .sum::<Scalar<E>>()
    }

    /// Validates a threshold keygen output:
    /// - every party agrees on the shared public key `P(0)` and the session id
    /// - all `n` shares lie on one degree-`(t-1)` polynomial.
    fn validate<E: Curve>(
        t: u16,
        n: u16,
        key_shares: &[KeyShare<E>],
        rng: &mut impl rand::RngCore,
    ) {
        assert_eq!(key_shares.len(), usize::from(n));

        let public_key = key_shares[0].share_point_0;
        for share in key_shares {
            assert_eq!(share.share_point_0, public_key);
        }

        // The party at 0-based position `k` holds the share at evaluation point `k + 1` (the 1-based party label).
        let all: Vec<(u16, Scalar<E>)> = (1..=n)
            .zip(key_shares.iter().map(|ks| ks.share_i))
            .collect();
        let subset: Vec<(u16, Scalar<E>)> =
            all.choose_multiple(rng, usize::from(t)).copied().collect();

        // The polynomial through the random `t`-subset must reproduce every other party's share
        for (label, share) in &all {
            if subset.iter().any(|(l, _)| l == label) {
                continue;
            }
            assert_eq!(interpolate_at(&subset, Scalar::from(*label)), *share);
        }

        // Constant term `p(0)` is the secret behind the public key.
        let secret = interpolate_at(&subset, Scalar::zero());
        assert_eq!(secret * Point::<E>::generator(), public_key);
    }

    fn keygen_works(t: u16, n: u16) {
        let mut rng = rand_dev::DevRng::new();

        let key_shares = round_based::sim::run_with_setup(
            core::iter::repeat_with(|| rng.fork()).take(n.into()),
            |i, party, rng| relaxed_key_generation::<_, _, Secp256k1>(rng, i, t, n, party, SID),
        )
        .unwrap()
        .expect_ok()
        .into_vec();

        validate(t, n, &key_shares, &mut rng);
    }

    #[test]
    fn simulation() {
        for &(t, n) in CASES {
            keygen_works(t, n);
        }
    }
}
