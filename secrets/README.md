# Secrets workflow (free, local-first)

## Windows (recommended): DPAPI secret store

Store secrets bound to your Windows user (not shareable across machines):

- Add/update a secret key:
  - `powershell -ExecutionPolicy Bypass -File scripts/secret_store.ps1 set OPENAI_API_KEY`
- Generate `.env` (no secrets printed):
  - `powershell -ExecutionPolicy Bypass -File scripts/gen_env.ps1`

This writes `.env` in the project root and ignores it via `.gitignore`.

## Linux/WSL: GPG-encrypted env file (optional)

If you prefer a single encrypted blob:

- Create: `gpg --symmetric --cipher-algo AES256 --output secrets/secrets.env.gpg your.env.plain`
- Generate: `bash scripts/gen_env.sh`

Notes:
- The decrypted env is written to `.env`; keep it out of git.
- For multiline secrets (private keys), prefer putting them in a file and referencing a path in `.env`.

