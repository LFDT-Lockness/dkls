# A sketch of the signing protocol in my head

Given curve $E$ of generator $G$ with prime order $q$, we start with ECDSA signing equation
```math
s = k^{-1} \cdot (z  + r \cdot x )
```
where message hash is $z = H(m)$, first coordinate $r = R.x = (k \cdot G).x$, secret key $x$, and secret nonce $k$.

For signing to work for t-of-n parties efficiently, we want the signature shares to be held additively:
```math
s = \sum_i s_i
```

## Secrets as additive shares

Luckily, the signing secret key $x$ coming out of the t-of-n key generation is Shamir shares held by the parties.
When $t$ parties $S$ are chosen to be signers, since the secret key is Lagrange interpolation of the $t$ Shamir shares, we can turn these $t$ Shamir shares into $t$ additive shares:
```math
x_i = \mathsf{lagrange}(S,0,i) \cdot p(i)
```
such that
```math
x = \sum_{i\in S} x_i = \mathsf{lagrange}(S,0,i) \cdot p(i)
```

The other secret nonce $k$, can also be done additively by having each signer sample locally $k_i$ such that:
```math
k = \sum_{i\in S} k_i
```

The other terms are all public: message hash $z$; $r$ from public $R = k \cdot G$.

## Secret inverse

The problem is the secret inverse $k^{-1}$ and secret multiplication $k^{-1} \cdot x$.

First, we turn secret inverse into public, by multiplying a random mask $\phi$ and its inverse:
```math
s = k^{-1} \cdot (\phi^{-1} \cdot \phi) \cdot (z + r \cdot x) = (k\cdot \phi)^{-1} \cdot (z\cdot \phi + r \cdot x\cdot \phi)
```
If $k\phi$ hides the nonce $k$, then we can open $k\phi$ for its inverse.

Let 
```math
u = k\cdot \phi, v = x\cdot \phi
```
then view $s$ additively,
```math
s = \sum_i u^{-1} \cdot (z\cdot \phi_i + r \cdot v_i) = \sum_i u^{-1}z \cdot \phi_i + u^{-1}r \cdot v_i) = \sum_i s_i
```
is our desired additive form.

## Secret multiplication

How do signers copmute $u$ and $v$?

We look at their additive shares:
```math
u = k\cdot \phi = (\sum_i k_i)\cdot (\sum_j \phi_j) = \sum_i k_i\cdot \phi_i + \sum_{i\neq j} k_i\cdot \phi_j
```
The diagonal terms $k_i\cdot \phi_i$ can be done locally at signer $i$, but the cross term $k_i\cdot \phi_j$ is a 2-party multiplication between signer $i$ and $j$.

How to do $k_i\cdot \phi_j$ privately?

Let Alice be signer $i$ with secret share $a = k_i$, Bob be signer $j$ with random mask share $b = \phi_j$.
We use Gilboa's multilication to addition trick, which relies on OT such that
```math
k_i\cdot \phi_j = a\cdot b = \alpha + \beta
```
with Alice holding $\alpha$, Bob $\beta$, but without learning each other's input.

Naively, we can do this iteratively with OT for each bit $b_l$ in $\phi_j$:
- Alice prepares two OT messages $m_0 = d$, $m_1 = a + d_l$, where $d_l$ is random
- Bob's choice bit $b_l$
- Bob receives $a \cdot b_l + d_l$

Then weighted sum by $2^l$, Alice holds
```math
\alpha = - \sum_l 2^l \cdot d_l = -\Delta
```
Bob holds
```math
\beta = \sum_l 2^l \cdot (a \cdot b_l + e_l) = a\cdot b + \Delta
```
and
```math
\alpha + \beta = a\cdot b = k_i\cdot \phi_j
```
is the correct additive share for the cross term product.

Naively this OT can be Endemic OT [MR19] based on elliptive curve, as offline phase, then online phase to calculate $\alpha$ and $\beta$.

This is inefficient, for the high number of OT instances requiring high number of curve operation.  Optimization is VOLE with OT extension, which caps the number of OT instances.

The above is the same for the other secret multiplication in $v$, with cross term $x_i\cdot \phi_j$.
Since $\phi_i$ is the shared factor within the two cross terms, we define Random VOLE to produce additive shares for both $k_i\cdot \phi_i$ and $x_i\phi_i$ in the same execution.

## Malicious setting

The above holds for semi-honest setting. For malicious setting, we will need to do more check.

TODO
