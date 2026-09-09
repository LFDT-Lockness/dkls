# OT Extension based on IKNP03

Why bother with IKNP's OT extension? Well SoftSpoken OT (Roy25) extension can be seen as follow up of IKNP with improvement on communication cost in the same Minicrypt model.

Technically, IKNP is also simpler: it uses only PRG and 1-of-2 OT, and without VOLE over polynomial-sized field. So it's "cleaner" to understand before moving up to SoftSpoken.

The consistency check of IKNP is based on KOS15, which has a flaw in its proof and corrected by Roy25.

## Parameters

- Number of columns in the matrix used internally, also viewed as computational security parameter: $\kappa$
- Statistical security parameter: $\lambda_s$
- Number of rows as delivered OTs: $m$
- Number of rows including consistency check: $m' = m + \kappa + \lambda_s$
- Field for interpreting each row as $\kappa$-bit string: $\mathbb{F}_{2^\kappa}$
- Alice as sender of $m$ OTs
- Bob as receiver of $m$ OTs

## Functions

- PRG for $\kappa$ columns and $m'$ rows: $G: \lbrace 0,1 \rbrace^\kappa \to \lbrace 0,1 \rbrace^{m'}$
- Hash with row-wise correlation robustness: $H: [m] \times \lbrace 0,1 \rbrace^\kappa \times \mathbb{X}$
- Non-programmable random oracle: $H_\Chi: \lbrace 0,1 \rbrace^\ast \to (\mathbb{F}_{2^\kappa})^{m'}$
- Base 1-of-2 OT such as [Endemic OT](endemic_ot.md): $F_\mathsf{BOT}$
- Bitwise concatenation $\parallel$
- Bitwise XOR $oplus$

## Round 1: setup - base 1-of-2 OTs for $\kappa$ columns

Bob as base OT sender:
- For each column $j \in [\kappa]$:
    - Sample a pair of bits as seeds: $k_0^j, k_1^j \leftarrow \lbrace 0,1 \rbrace^2$
    - Send $(k_0^j, k_1^j)$ via $F_\mathsf{BOT}$

Alice as base OT receiver:
- Sample $\kappa$ choice bits for each seed: $s \in \lbrace 0,1 \rbrace^\kappa$
- For each column $j \in [\kappa]$:
    - Receive $k_{s_j}^j$ via $F_\mathsf{BOT}$

## Round 2: $r$ choice bits

Bob as OT extension receiver:
- Sample $m'$ choice bits $r'$
    - For $m$ rows: $r \leftarrow \lbrace 0,1 \rbrace^m$
    - For $m'-m$ rows: $\tilde{r} \leftarrow \lbrace 0,1 \rbrace^{m'-m}$
    - Let $r' = r \parallel \tilde{r}$
- For each column $j \in [\kappa]$:
    - Generate private column $t^j = G(k_0^j)$
    - Commit choice $r'$ as column: $u^j = G(k_0^j) \oplus G(k_1^j) \oplus r'$
- Send commitment $(u^1,\cdots,u^\kappa)$ to Alice
- Derive Fiat-Shamir challenge after commitment: $\Chi = H_\Chi(u^1,\cdots,u^\kappa)$
- Compute consistency check arguments in field $\mathbb{F}_{2^\kappa}$
    - $x = \sum_{i \in [m']}\,\Chi_i \cdot r'_i$
    - $t = \sum_{i \in [m']}\,\Chi_i \cdot t_i$
- Send $(x,t)$ to Alice for consistency check

## Round 3: $m'$ rows

Alice as OT extension sender:
- For each column $j \in [\kappa]$:
    - Compute column $q^j = (s_j \cdot u^j) \oplus G(k_{s_j}^j)$
- Derive Fiat-Shamir challenge: $\Chi = H_\Chi(u^1,\cdots,u^\kappa)$
- Check using linear combination of rows $q_i:
    - Abort if $\sum_{i \in [m']}\,\Chi_i \cdot q_i \neq t \oplus x \cdot s$
- Drop rows $m + 1,\cdots, m'$

## Output: $m$ OTs

Alice:
- For each row $i in [m]$, output
    - $m_{i,0} = H(i, q_i)
    - $m_{i,1} = H(i, q_i \oplus s)

Bob:
- For each row $i in [m]$, output
    - $m_{i,r_i} = H(i, t_i)$
