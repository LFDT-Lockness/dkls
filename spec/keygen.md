# Threshold (i.e.,  t-out-of-n) relaxed distributed key generation

## Parameters
- Security parameter $\lambda = 128$

## Functions
- [Unambiguous encoding](https://docs.rs/udigest/latest/udigest/encoding/index.html): $\mathsf{encode}_{\mathsf{tag}}(\cdot) \to \lbrace 0,1 \rbrace ^*$
- SHA-256 $H(\cdot) \to \lbrace 0,1 \rbrace ^{2\lambda}$

## Input

- Party index $i$, $1 \le i \le n$
- Threshold parameter $t$, $2 \le t \lt n$
- Number of signers $n \ge 2$
- Context id $\mathsf{sid}$
- Curve $\mathbb{E}$ with generator $G$ of prime order $q$

## Round 1: commit

Party $i$:
- Sample $a_{i,0}, a_{i,1}, \cdots, a_{i,t-1} \leftarrow \mathbb{Z}_q^t$ as coefficients of degree $(t-1)$ polynomial $p_i$
- Compute $n$ subshares for $[n]$ as $p_i(1), p_i(2), \cdots, p_i(n)$
- Compute $t$ subshare curve points for $[0, t-1]$ as $\overrightarrow{P}_i = (P_i(0), P_i(1),\cdots, P_i(t-1))$ where  $P_i(j) = p_i(j)\cdot G$
- Broadcast commit subshare curve points:
    - Sample nonce $u_i \leftarrow \lbrace 0,1 \rbrace ^{2\lambda}$
    - Let committed points be $m_i := \overrightarrow{P}_i$
    - Let $\mathsf{tag} := ``dkls23.keygen.committed.points"$
    - Compute $V_i \leftarrow H(\mathsf{encode}_\mathsf{tag}(\lbrace \mathsf{sid}, \mathsf{committer}: i, m_i, \mathsf{nonce}: u_i \rbrace ))$
    - Send $(\mathsf{CommitPoints}, V_i)$ to every other parties
- 2-party commit subshares
    - For $j \in [n]\setminus\lbrace i\rbrace$
        - Sample nonce $u^\prime_{i\rightarrow j} \leftarrow \lbrace 0,1\rbrace ^{2\lambda}$
        - Let committed subshare be $m^\prime_{i\rightarrow j} := p_i(j + 1)$
        - Let $\mathsf{tag} := ``dkls23.keygen.committed.subshare"$
        - $V^\prime_{i\rightarrow j} \leftarrow H(\mathsf{encode}_\mathsf{tag}(\lbrace \mathsf{sid}, \mathsf{committer}: i, \mathsf{receiver}: j, m^\prime_{i\rightarrow j} , \mathsf{nonce}: u^\prime_{i\rightarrow j} \rbrace ))$
        - Send $(\mathsf{CommitSubshare}, V^\prime_{i\rightarrow j})$ to party $j$

## Round 2: decommit

Party $i$:
- Receive all broadcast commitments $\lbrace V_j: j \neq i\rbrace$ and 2-party commitments $\lbrace V^\prime_{j\rightarrow i}: j \neq i\rbrace$
- Echo and open for broadcast commitments :
    - Let $\mathsf{tag} := ``dkls23.keygen.echo.commitments"$
    - Compute echo digest $h_i = H(\mathsf{encode}_\mathsf{tag}(\lbrace \mathsf{commitments}: (V_1, \cdots, V_n) \rbrace))$
    - Send $(\mathsf{EchoDigestAndDecommitPoints}, h_i, m_i, u_i)$ to every other party
- Open for 2-party commitments
    - For $j \in [n]\setminus\lbrace i\rbrace$
        - Send $(\mathsf{DecommitSubshare}, m^\prime_{i\rightarrow j}, u^\prime_{i\rightarrow j})$ to party $j$

## Round 3: verify

Party $i$:
- Receive $\lbrace h_j, m_j, u_j, m^\prime_{j\rightarrow i}, u^\prime_{j\rightarrow i}: j\neq i\rbrace$ from every party $j \neq i$
    - Parse each received $m_j$ as $(P_j(0), P_j(1), \cdots, P_j(t-1))$ from party $j$
    - Parse each received $m^\prime_{j\rightarrow i}$ as $p_j(i+1)$ from party $j$
- Echo agreement for broadcast commitments:
    - Abort if exists $h_j$ such that $h_j \neq h_i$: 
        - Send $(\mathsf{Abort})$ to every other party
        - Go to Output
- Binding
    - For $j \in [n]\setminus \lbrace i\rbrace$:
        - Abort if broadcast commitment $V_j \neq H(\mathsf{encode}(\lbrace \mathsf{sid}, \mathsf{committer}: j, m_j, \mathsf{nonce}: u_j\rbrace))$: 
            - Send $(\mathsf{Abort})$ to every other party
            - Go to Output
        - Abort if 2-party commitment $V^\prime_{j\rightarrow i} \neq H(\mathsf{encode}(\lbrace \mathsf{sid}, \mathsf{committer}: j, \mathsf{receiver}: i, m^\prime_{j\rightarrow i}, \mathsf{nonce}: u^\prime_{j\rightarrow i} \rbrace))$
            - Send $(\mathsf{Abort})$ to every other party
            - Go to Output
- Sum to $t$ share curve points for $k \in [0, t-1]$: $P(k) = P_0(k) + P_1(k) + \cdots P_{n-1}(k)$
- Sum to share: $s_i := p(i+1) = p_0(i+1) + p_1(i+1) + \cdots + p_{n-1}(i+1)$
- Compute share curve point $P_i = s_i \cdot G$
- Compute expected share curve point $Q$ as follows:
    - If $i + 1 \in [t-1]$:
        - $Q \leftarrow P(i + 1)$
    - Else build from lagrange coefficient of $t$ curve points:
        - Form $t$ 1-based labels corresponding: $S = [t-1] \cup \lbrace i+1\rbrace$
        - Compute $Q \leftarrow \lambda_{i+1}^{-1} \cdot (P(0) - (\lambda_1 \cdot P(1) + \lambda_2 \cdot P(2) + \cdots + \lambda_{t-1} P(t-1)))$ where
            - $\lambda_k := \mathsf{lagrange}(S, k, 0) \in \mathbb{Z}_q$
            - $\mathsf{lagrange}(S, k, x) := \prod_{l\in S, l \neq k} (x-l) \cdot (k-l)^{-1} \in \mathbb{Z}_q$
- Abort if $P_i \neq Q$
    - Send $(\mathsf{Abort})$ to every other party
    - Go to Output
- Send $(\mathsf{Ok})$ to every other party

## Output

Party $i$:
- If received $(\mathsf{Abort})$ from any party, or sent $(\mathsf{Abort})$ to any party:
    - Output $(\mathsf{Abort})$
    - Halt from this session $\mathsf{sid}$
- If sent $(\mathsf{Ok})$ to every other party, and received $(\mathsf{Ok})$ from every other party:
    - Output $(\mathsf{KeyShare}, P(0), s_i)$
