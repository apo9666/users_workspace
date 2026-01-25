# Autenticação e MFA

Este documento descreve os tokens JWT, fluxos de login/MFA e os claims

## Tokens JWT

### 1) Access Token (normal)
- typ: "access"
- sub: user_id
- exp: curto
- scope: []

### 2) Access Token (elevado - step-up)
- typ: "access"
- sub: user_id
- scope: ["mfa:manage"]
- exp: 5-10 min

### 3) MFA Token
- typ: "mfa_registration" | "mfa_verification"
- sub: user_id
- jti: uuid
- exp: 2-5 min

## Rotas

### Login
- POST `/login`
  - sucesso sem MFA -> access + refresh
  - MFA requerido -> mfa_verification 

### Access TOTP
- POST `/totp/verify`
  ```json
  {
    "mfa_verification": "...",
    "code": "670059"
  }
  ```
  - mfa_verification -> access + refresh

### Access WebAuthn
- POST `/webauthn/authentication/start`
  ```json
  {
    "mfa_verification": "...",
  }
  ```
- POST `/webauthn/authentication/finish`
  ```json
  {
    "mfa_verification": "...",
  }
  ```
  - mfa_verification -> access + refresh

### Gerenciamento de MFA (exigem access elevado)
- GET `/mfa`
Header: Bearer access
  ```json
  {
    "mfa_registration": "...",
    "allowed_methods": ["totp", "webauthn"],
    "expires_in": 180
  }
  ```

### Access elevated TOTP
- POST `/totp/verify`
  ```json
  {
    "mfa_registration": "...",
    "code": "670059"
  }
  ```
  - mfa_registration -> access ["mfa:manage"]

### Access WebAuthn
- POST `/webauthn/authentication/start`
  ```json
  {
    "mfa_registration": "...",
  }
  ```
- POST `/webauthn/authentication/finish`
  ```json
  {
    "mfa_registration": "...",
  }
  ```
  - mfa_registration -> access ["mfa:manage"]  


### Register TOTP
- POST `/totp/registration/start`
  - access ["mfa:manage"]
- POST `/totp/registration/finish`
  - access ["mfa:manage"]
- DELETE `/totp`
  - access ["mfa:manage"]

### Register - WebAuthn
- POST `/webauthn/registration/start`
  - access ["mfa:manage"]
- POST `/webauthn/registration/finish`
  - access ["mfa:manage"]
- GET `/webauthn`
  - access ["mfa:manage"]
- GET `/webauthn/{id}`
  - access ["mfa:manage"]
