# phient — architecture

## The modules and what they use

```mermaid
graph TB
    subgraph crate["phient"]
        m_baseline["baseline"]
        m_code["code"]
        m_lineup["lineup"]
        m_operator["operator"]
        m_prose["prose"]
        m_relate["relate"]
        m_study["study"]
    end
    m_baseline --> m_prose
    m_code --> m_prose
    m_code --> m_relate
    m_code --> m_study
    m_lineup --> m_operator
    m_lineup --> m_study
    m_operator --> m_relate
    m_prose --> m_relate
    m_relate --> m_operator
    m_relate --> m_prose
    X_phical["phical (crate)"]
    m_lineup -.-> X_phical
    m_operator -.-> X_phical
    m_relate -.-> X_phical
    X_phiclm["phiclm (crate)"]
    m_code -.-> X_phiclm
    X_phiecs["phiecs (crate)"]
    m_code -.-> X_phiecs
    m_lineup -.-> X_phiecs
    m_operator -.-> X_phiecs
    m_prose -.-> X_phiecs
    m_relate -.-> X_phiecs
    X_phistd["phistd (crate)"]
    m_lineup -.-> X_phistd
    m_operator -.-> X_phistd
    m_prose -.-> X_phistd
    m_relate -.-> X_phistd
    X_rayon["rayon (crate)"]
    m_lineup -.-> X_rayon
```

## Modules, one diagram each

- [baseline](../../docs/phient/baseline.md)
- [code](../../docs/phient/code.md)
- [lineup](../../docs/phient/lineup.md)
- [operator](../../docs/phient/operator.md)
- [prose](../../docs/phient/prose.md)
- [relate](../../docs/phient/relate.md)
- [study](../../docs/phient/study.md)

What is left to do: [TODO.md](TODO.md)
