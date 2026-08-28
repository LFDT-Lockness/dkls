# Endemic 1-of-2 OT [MR19]

## Parameters

As in DKLs:
- Security parametr $\lambda = 128$
- Ouput length $\ell = \lambda$ (length of OT string to seed OT extension's PRG)
- Curve $\mathbb{E}$ with generator $G$ of prime order $q$ with $\log_2(q) \ge 2\lambda$

## Functions

- Random oracle for a point of unknown log: $H: \langle G \rangle \to \langle G \rangle$
    - Instantiated by hashing to curve RFC 9380
- Random oracle: $H': \langle G \rangle \to \lbrace 0,1 \rbrace^\ell$
    - Instantiated by $\mathsf{SHA256}(\mathsf{encode}(\cdot))$ truncated to $\ell$ bits, where $\mathsf{encode}(\cdot): \langle G \rangle \to \lbrace 0,1 \rbrace^\ast$ encodes a curve point according to its coordinates

## Input

- Receiver:
    - choice bit $c \in \lbrace 0, 1 \rbrace$
- Sender:

- Common:
    - Session id $\mathsf{sid}$

## Round 1

Bob (Receiver):
- Sample a point for choice $1-c$ as $R_{1-c} \leftarrow G$
- Sample a scalar $a \leftarrow \mathbb{Z}_q$
- Compute the point for choice $c$ as $R_c = a \cdot G - H(\mathsf{encode}(\mathsf{sid}, c), R_{1-c})$
- Send $(R_0, R_1)$ to Alice

Alice (Sender):
- Sample random scalars $b_0, b_1 \leftarrow \mathbb{Z}_q$
- Compute points in the cyclic group of $\langle G \rangle$:
    - $B_0 = b_0 \cdot G$ 
    - $B_1 = b_1 \cdot G$
- Send $(B_0, B_1)$ to Bob

## Output

Bob:
- Compute $\rho_c = H'(\mathsf{encode}(\mathsf{sid}), a \cdot B_c)$
- Output $(c, \rho_c)$

Alice:
- For $j \in [0,1]$
    - Compute $M_j = R_j + H(\mathsf{encode}(\mathsf{sid}, j), R_{1-j})$
    - Compute $\rho_j = H'(\mathsf{encode}(\mathsf{sid}), b_j \cdot M_j)$
- Output $(\rho_0, \rho_1)$
