## 1. Documentation Polish
- [x] 1.1 Review `openspec/specs` wording for consistency (SHALL/MUST, error naming)
- [x] 1.2 Align `README.md` quickstart and configuration with current env vars
- [x] 1.3 Update `docs/configuration.md` with `POBLYSH_CRYPTO_KEY` details (+ link to runbook)

## 2. Local Crypto Rotation Runbook
- [x] 2.1 Add `docs/runbooks/local-crypto-rotation.md` with two paths:
      Preferred (local/dev): recreate connections/tokens; Advanced: preserve tokens with future maintenance command
- [x] 2.2 Cross‑link runbook from `README.md` and `docs/configuration.md`

## 3. Tech/Crate Selections (for review)
- [x] 3.1 Confirm crate versions: `aes-gcm 0.10.3`, `zeroize 1.8.1`, `base64 0.22.1`, optional `hkdf 0.12.4`
- [x] 3.2 Validate edition compat (2024) and MSRV used by crates

## 4. Spec Follow-up (post-encryption merge)
- [x] 4.1 Add/modify `config` and `crypto` specs to reference env var, startup validation, and error mapping
- [x] 4.2 Add QA validation notes to `qa.md` and resolve any inconsistencies

