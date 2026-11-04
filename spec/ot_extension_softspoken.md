# OT Extension based on SoftSpoken

## Parameters

- Computational security parameter: $\lambda_c$
- Statistical security parameter: $\lambda_s$
- Number of rows as delivered OTs: $m$
- Number of rows including consistency check: $m' = m + \lambda_c + \lambda_s$
- Field for interpreting each row as $\lambda_c$-bit string: $\mathbb{F}_{2^{\lambda_c}}$
- Alice as sender of $m$ OTs
- Bob as receiver of $m$ OTs
- SoftSpoken
    - Small VOLE field as matrix entry: 
        - Length parameter $k$ ($k=2$ for DKLs)
        - Field: $\mathbb{F}_{2^k}$
    - number of VOLE columns: $n_c = \lambda_c / k$

## Functions

- PRG with $\lambda_c$-bit seed for $m'$ rows: $G: \lbrace 0,1 \rbrace^{\lambda_c} \to \lbrace 0,1 \rbrace^{m'}$
- Hash with row-wise correlation robustness: $H: [m] \times \lbrace 0,1 \rbrace^{n_c} \times \mathbb{X}$
- Non-programmable random oracle: $H_\Chi: \lbrace 0,1 \rbrace^\ast \to (\mathbb{F}_{2^{\lambda_c}})^{m'}$
- Bitwise concatenation $\parallel$
- Bitwise XOR $oplus$
- Base, all-but-1 OT [All-but-1 OT](allbut1_ot.md): $F_\mathsf{AOT}$
- GGM tree as function $\rho^j: \mathbb{F}_{2^k} \to \lbrace 0,1 \rbrace^{\lambda_c}$ for column $j$

## Round 1: Setup

Between Alice and Bob :
- Run all-but-one OT $F_\mathsf{AOT}(2^k, n_c, k)$:
    - For each column $j$:
        - Alice gets 
            - Uniform choice value $\Delta_j \in \mathbb{F}_{2^k}$
            - All-but-one seeds $\rho^j(x)$ for every $x \neq \Delta_j$ 
        - Bob gets all seeds $\rho^j(x)$ for every $x \in \mathbb{F}_{2^k}$

## Round 2: Bob

Bob:
- Sample $m'$ choice bits $r'$
    - For $m$ rows: $r \leftarrow \lbrace 0,1 \rbrace^m$
    - For $m'-m$ rows: $\tilde{r} \leftarrow \lbrace 0,1 \rbrace^{m'-m}$
    - Let $r' = r \parallel \tilde{r}$
- For each column $j \in [n_c]$:
    - Generate private column 
        - $v^j = - \sum_{x=\mathbb{F}_{2^k}} x \cdot G(\rho^j(x))$
        - $u^j = \sum_{x=\mathbb{F}_{2^k}} G(\rho^j(x))$
    - Commit choice $r'$ as column $C^j = u^j \oplus r'$
- Send commitment $C^1, \cdots, C^{n_c}$ to Alice
- Derive Fiat-Shamir challenge: $\Chi = H_\Chi(C^1, \cdots, C^{n_c})$
- Compute consistency check arguments in field $\mathbb{F}_{2^\lambda_c}$
    - $x = \sum_{i \in [m']}\,\Chi_i \cdot r'_i$
    - $t = \sum_{i \in [m']}\,\Chi_i \cdot v_i$
- Send $(x,t)$ to Alice for consistency check

## Round 3: Alice

Alice:
- For each column $j \in [n_c]$
    - Compute column $w^j = (\Delta_j \cdot C^j) \oplus \sum_{x \neq \Delta_j} (\Delta_j - x) \cdot G(\rho^j(x))$
- Check consistency: 
    - Derive Fiat-Shamir challenge: $\Chi = H_\Chi(C^1, \cdots, C^{n_c})$
    - Abort if $\sum_{i \in [m']} \Chi_i \cdot w_i \neq t \oplus x \cdot \Delta$
- Drop rows $m + 1,\cdots, m'$

## Output

Alice:
- For each row $i \in [m]$, output
    - $m_{i,0} = H(i, w_i)$
    - $m_{i,1} = H(i, w_i - \Delta)$

Bob:
- For each row $i \in [m]$, output
    - $m_{i,r_i} = H(i, v_i)$

