# Carteira de Investimentos

Aplicação web fullstack para cadastrar e acompanhar ativos de investimento (nome e valor unitário), com cadastro/login de pessoas usuárias.

## O que o projeto faz

- Permite criar, listar e atualizar ativos de investimento (ex: Bitcoin, Ethereum) via API.
- Permite que uma pessoa usuária se cadastre e faça login (se o usuário não existir, ele é criado automaticamente no primeiro login).
- Mantém a sessão autenticada através de um cookie assinado contendo um token JWT.
- Exibe uma página inicial simples que cumprimenta a pessoa logada, ou redireciona para o login.

## Tecnologias usadas

- **Rust** com **Axum** — servidor web e roteamento.
- **SQLx** + **PostgreSQL** — persistência de dados (ativos e usuários).
- **Askama** — templates HTML (página de login).
- **jwt-simple** — geração/validação de tokens JWT para autenticação.
- **password-auth** — hash e verificação segura de senhas.
- **axum-extra** — cookies assinados (`CookieJar`).
- **Docker Compose** — sobe o banco PostgreSQL localmente.
- **insta** — snapshot testing dos retornos da API.
- **tracing** — logs estruturados.

## Como executar a aplicação

1. Suba o banco de dados:
   ```
   docker compose up -d
   ```
2. Configure a variável de ambiente (já existe um `.env` de exemplo com):
   ```
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
   ```
3. Rode as migrações e inicie a aplicação:
   ```
   cargo run
   ```
4. Acesse `http://localhost:3000` no navegador. Você será redirecionado para `/login`.

### Rotas principais

| Método | Rota          | Descrição                                   |
|--------|---------------|----------------------------------------------|
| GET    | `/`           | Página inicial (requer login)                |
| GET    | `/login`      | Formulário de login/cadastro                 |
| POST   | `/login`      | Autentica (ou registra) a pessoa usuária      |
| GET    | `/api/assets` | Lista os ativos cadastrados                   |
| POST   | `/api/assets` | Cria um novo ativo (requer header `Admin`)    |
| PATCH  | `/api/assets` | Atualiza um ativo existente (requer `Admin`)  |

## Melhoria implementada

Foquei em **aumentar a cobertura de testes automatizados** do projeto, sem alterar o comportamento existente:

- `repository.rs`: testes de criação/listagem de ativos, atualização parcial (só nome, só valor), atualização de ativo inexistente, cadastro/busca de usuário e violação de usuário duplicado.
- `auth/user.rs`: teste de geração e validação do token JWT, registro de usuário, tentativa de registro com nome duplicado, autenticação com sucesso, senha errada e usuário inexistente.
- `error.rs`: teste que garante que cada `AppError` é convertido no código HTTP correto (400, 401, 404).
- `routes/api.rs`: teste de criação de ativo com nome duplicado, atualização de ativo inexistente e atualização parcial via handler da API.
- `routes/frontend.rs`: teste de login registrando um novo usuário, login com senha errada, e da página inicial (`/`) redirecionando quando não autenticado ou cumprimentando quando autenticado.

## Como testar minha versão

Com o banco de dados rodando (`docker compose up -d`), execute:

```
cargo test
```

Os testes usam `#[sqlx::test]`, que cria automaticamente um banco de testes isolado (com as migrações aplicadas) para cada teste, então não é necessário preparar dados manualmente antes de rodar.

## O que aprendi durante o desafio

- Como estruturar uma aplicação Axum em módulos (rotas, repositório, autenticação, erros) mantendo cada camada com responsabilidade única.
- Como usar `FromRequestParts` para criar extractors customizados (`Admin`, `User`, `Repository`) que simplificam as assinaturas dos handlers.
- Como testar handlers Rust/Axum diretamente (sem precisar de um servidor HTTP rodando), usando `#[sqlx::test]` para isolar o banco de dados em cada teste.
- Como o SQLx verifica as queries em tempo de compilação e como usar fixtures (`fixtures("...")`) para popular dados de teste.
- Como o fluxo de autenticação combina hashing de senha (`password-auth`), cookies assinados e JWT para manter uma sessão segura sem guardar estado no servidor.
