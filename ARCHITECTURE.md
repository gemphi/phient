# phient — architecture

## The modules and what they use

```mermaid
graph TB
    subgraph crate["phient"]
        m_baseline["baseline"]
        m_code["code"]
        m_eval["eval<br/>the Evaluator · oscillator"]
        m_generate["generate<br/>the decoders, the guide, the persona prior<br/>learn (its scorer), compose (better/worse, river)"]
        m_lineup["lineup"]
        m_memory["memory<br/>the 16-layer interaction log"]
        m_metrics["metrics<br/>the harness, benchmarks, kernel"]
        m_net["net<br/>the lexicon as a function space, its reader and scorer"]
        m_operator["operator"]
        m_prose["prose"]
        m_relate["relate"]
        m_scribe["scribe"]
        m_study["study"]
        m_train["train<br/>trainer, topdown, chunker, conception, sources, roles, grounding<br/>kinds, synthetic, curriculum, correction, envision, synthesis, lifelong"]
    end
    m_baseline --> m_prose
    m_generate --> m_eval
    m_generate --> m_metrics
    m_generate --> m_net
    m_generate --> m_train
    m_code --> m_prose
    m_code --> m_relate
    m_code --> m_study
    m_lineup --> m_operator
    m_train --> m_memory
    m_metrics --> m_train
    m_metrics --> m_eval
    m_metrics --> m_baseline
    m_net --> m_metrics
    m_net --> m_memory
    m_lineup --> m_study
    m_operator --> m_relate
    m_prose --> m_relate
    m_relate --> m_operator
    m_relate --> m_prose
    m_scribe --> m_lineup
    m_scribe --> m_study
    X_phical["phical (crate)"]
    m_generate -.-> X_phical
    m_lineup -.-> X_phical
    m_net -.-> X_phical
    m_operator -.-> X_phical
    m_relate -.-> X_phical
    X_phiclm["phiclm (crate)"]
    m_code -.-> X_phiclm
    m_scribe -.-> X_phiclm
    X_phiecs["phiecs (crate)"]
    m_generate -.-> X_phiecs
    m_code -.-> X_phiecs
    m_lineup -.-> X_phiecs
    m_train -.-> X_phiecs
    m_metrics -.-> X_phiecs
    m_eval -.-> X_phiecs
    m_eval --> m_memory
    m_net -.-> X_phiecs
    m_operator -.-> X_phiecs
    m_prose -.-> X_phiecs
    m_relate -.-> X_phiecs
    X_phistd["phistd (crate)"]
    m_generate -.-> X_phistd
    m_lineup -.-> X_phistd
    m_train -.-> X_phistd
    m_metrics -.-> X_phistd
    m_net -.-> X_phistd
    m_operator -.-> X_phistd
    m_prose -.-> X_phistd
    m_relate -.-> X_phistd
    X_phiton["phiton (crate)"]
    m_generate -.-> X_phiton
    m_lineup -.-> X_phiton
    m_train -.-> X_phiton
    m_metrics -.-> X_phiton
    m_eval -.-> X_phiton
    m_memory -.-> X_phiton
    m_net -.-> X_phiton
    m_operator -.-> X_phiton
    X_rayon["rayon (crate)"]
    m_lineup -.-> X_rayon
    m_train -.-> X_rayon
    m_metrics -.-> X_rayon
```

## Modules, one diagram each

- [baseline](../../docs/phient/baseline.md)
- [code](../../docs/phient/code.md)
- [eval](../../docs/phient/eval.md)
- generate: [generate](../../docs/phient/generate.md) · [learn](../../docs/phient/learn.md) · [compose](../../docs/phient/compose.md)
- [lineup](../../docs/phient/lineup.md)
- [memory](../../docs/phient/memory.md)
- [metrics](../../docs/phient/metrics.md) · [nonlinear](../../docs/phient/nonlinear.md) · [data](../../docs/phient/data.md)
- [net](../../docs/phient/net.md)
- [operator](../../docs/phient/operator.md)
- [prose](../../docs/phient/prose.md)
- [relate](../../docs/phient/relate.md)
- [scribe](../../docs/phient/scribe.md)
- [study](../../docs/phient/study.md)
- train: [kinds](../../docs/phient/kinds.md) · [synthetic](../../docs/phient/synthetic.md) · [curriculum](../../docs/phient/curriculum.md) · [correction](../../docs/phient/correction.md) · [envision](../../docs/phient/envision.md) · [synthesis](../../docs/phient/synthesis.md) · [lifelong](../../docs/phient/lifelong.md) · [trainer](../../docs/phient/trainer.md) · [topdown](../../docs/phient/topdown.md) · [chunker](../../docs/phient/chunker.md) · [conception](../../docs/phient/conception.md) · [sources](../../docs/phient/sources.md) · [roles](../../docs/phient/roles.md) · grounding ([docs/phiano/core.md](../../docs/phiano/core.md))

What is left to do: [TODO.md](TODO.md)
