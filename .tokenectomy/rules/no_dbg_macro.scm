;; @id: CORP_SEC_001
;; @name: no-dbg-macro
;; @severity: error
;; @message: dbg!(...) macro detected in production code. Use structured tracing instead.
;; @fix_hint: Remove dbg! or use tracing::debug!
;; @languages: rust

(macro_invocation
  macro: (identifier) @name (#eq? @name "dbg")) @match
