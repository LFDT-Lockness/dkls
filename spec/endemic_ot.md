# Endemic 1-of-2 OT [MR19]

## Parameters

- Security parametr $\lambda$
- Ouput length $\ell$
- Curve $\mathbb{E}$ with generator $G$ of prime order $q$

## Functions

- Random oracle on point: $H: \langle G \rangle \to \langle G \rangle$
- Random oracle on: $H': \langle G \rangle \to \lbrace 0,1 \rbrace^\ell$

## Input

- Receiver:
    - choice bit $c \in \lbrace 0, 1 \rbrace$
- Sender:

- Common:
    - Context id $\mathsf{sid}$

## Round 1

Bob (Receiver):
- Sample a point for choice $1-c$ as $r_{1-c} \leftarrow G$
- Sample a scalar $a \leftarrow \mathbb{Z}_q$
- Compute the point for choice $c$ as $r_c = a \cdot G - H(\mathsf{encode}(\mathsf{sid}, c), r_{1-c})$
- Send $(r_0, r_1)$ to Alice

Alice (Sender):
- Sample random scalars $b_0, b_1 \leftarrow \mathbb{Z}_q$
- Compute points from the cyclic group of $G$:
    - $B_0 = b_0 \cdot G$ 
    - $B_1 = b_1 \cdot G$
- Send $(B_0, B_1)$ to Bob

## Output

Bob:
- Compute $\rho_c = H'(\mathsf{encode}(\mathsf{sid}), a \cdot B_c)$
- Output $(c, \rho_c)$

Alice:
- For $j \in [0,1]$
    - Compute $M_j = r_j + H(\mathsf{encode}(\mathsf{sid}, j), r_{1-j})$
    - Compute $\rho_j = H'(\mathsf{encode}(\mathsf{sid}), b_j \cdot M_j)$
- Output $(\rho_0, \rho_1)$
