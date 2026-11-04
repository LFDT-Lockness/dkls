# All-but-one OT in Roy22, $F_\mathsf{AOT}$

## Parameters
- Domain size $N$ (SoftSpoken uses $N=2^k$; DKLs uese $N=4$)
- Number of base OTs $k$ (GGM tree depth)
- OT Sender as Bob
- OT Receiver as Alice
- Node and table entry width in bits $\lambda_c = 128$

## Functions
- Base 1-of-2 OT $F_\mathsf{OT}$ (e.g. [Endenmic OT](./endemic_ot.md))
- PRG for tree growth $G: \lbrace 0,1 \rbrace^{\lambda_c} \to \lbrace 0,1 \rbrace^{2\lambda_c}$
    - Two halves: $G_0, G_1$
- PRG for leaf post-processing: $G': \lbrace 0,1\rbrace^{\lambda_c} \to \lbrace 0,1 \rbrace^{2\lambda_c} \times \lbrace 0,1\rbrace^{\lambda_c}$
    - Two halves: $G'_0, G'_1$
- Sender's table $F: \lbrace 0,1 \rbrace \to \lbrace 0,1\rbrace^{\lambda_c}$

## Round 1: Setup

Between Alice and Bob:
- Run $k$ independent base OT $F_\mathsf{OT}$ with Bob as sender and Alice as receiver:
    - For every level $i \in [k]$
        - Bob gets messages $F_i(0), F_i(1)$
        - Alice gets choice bit $x^*_i \in \lbrace 0,1 \rbrace$, message $F_i(c^-_i)$ where $c^-_i = 1 - x^*_i$


## Round 2: Bob

Bob:
- Build punctured PRF
    - Let $s^1_c = F_0(c)$ for $c \in \lbrace 0,1 \rbrace$, base OT $0$ at level 1
    - For level $i \in [k-1]$:
        - For 2 branches per level $c \in \lbrace 0,1 \rbrace$
            - Compute $s^{i+1}_{2y+c} = G_c(s^i_y)$ for $y \in [2^i]$
            - Compute $t^i_c = \oplus_{y\in [2^i]} s^{i+1}_{2y+c}$
            - Send $F_i(c) \oplus t^i_c$ to Alice
- Prove punctured PRF
    - Send 
        - $\tilde{s} = H(\parallel_{y \in [2^k]}) G'_0(s^k_y)$
        - $\tilde{t} = \oplus_{y \in [2^k]} G'_0(s^k_y)$

## Round 3: Alice

Alice
- Evaluate punctured PRF
    - Let $s^1_{c^-_0} = F_0(c^-_0)$ where $c^-_0 = 1 - x^*_0$, puncture at $y^*_1 = x^*_0$
    - For level $i = [k-1]$
        - Receive $R = F_i(c^-_i) \oplus t^i_{c^-_i}$ from Bob
        - Compute $S = \oplus_{y \neq y^*_i} s^{i+1}_{2y+c}$ where $s^{k+1}_{2y+c} = G_c(s^i_y)$ for $y \neq y^*_i$
        - Compute $s^{k+1}_{2y^*_i+c^-_i} = R \oplus F_i(c^-_i) \oplus S$
        - Compute $y^*_{i+1} = 2 \cdot y^*_i + x^*_i$
    - Arrange in base 2: $x^* = y^*_k = (x^*_0, x^*_1, \dots, x^*_{k-1})_2$
- Verify punctured PRF
    - $G'_0(s^k_{x^*}) = \tilde{t} \oplus \oplus_{y \neq x^*} G'_0(s^k_y)$
    - Check: abort if $H(\parallel_{y\in [2^k]} G'_0(s^k_y)) \neq \tilde{s}$

## Output

Bob:
- $F(y) = G'_1(s^k_y)$ for every $y \in [2^k]$

Alice:
- $x^*$
- $F_y) = G'_1(s^k_y)$ for every $y \neq x^*$
