# LinkShort

A URL shortener with click analytics, built from scratch in Rust as a learning project.

Paste a long URL, get a short link. Every click is tracked with timestamp, IP, referrer, and user agent. Manage everything from a clean dashboard.

## Why I built this

I wanted to learn Rust beyond toy examples. A URL shortener is small enough to finish but touches real concerns: async HTTP servers, database queries, authentication, middleware, and deployment. Building it as a full-stack app forced me to deal with template rendering, state management, and production issues like idempotent migrations and Docker multi-stage builds.

## Features

- Shorten any URL with auto-generated or custom short codes
- 302 redirects with async click tracking (non-blocking)
- Per-link analytics: total clicks, daily breakdown, recent click log
- JWT authentication with bcrypt password hashing
- Server-rendered HTML templates with vanilla JavaScript
- Mobile responsive layout
- Dockerized with docker-compose for one-command local setup
- Configurable for production deployment (Railway, Fly.io, etc.)

## Architecture

The app follows a layered structure common in web backends:

```
Request
  |
  v
Router (axum)
  |
  v
Handlers ---- extract Auth from JWT
  |
  v
Models ------- run queries via sqlx
  |
  v
PostgreSQL
```

Redirects are the hot path. When someone visits a short link, the handler looks up the original URL, fires off a background task (via `tokio::spawn`) to log the click, and immediately returns a 302 redirect. The click tracking never blocks the redirect response.

Authentication uses stateless JWTs. The `AuthUser` extractor pulls the token from the `Authorization` header, validates it against the signing secret, and injects the user ID into the handler. No session storage needed.

Templates use Askama, which compiles HTML to Rust code at build time. The frontend is plain HTML with Tailwind CSS (via CDN) and vanilla JS for API calls. No build step, no npm, no bundler.

## Project structure

```
src/
  main.rs             -- server startup, route wiring, migrations
  config.rs           -- env var loading
  db.rs               -- postgres connection pool
  errors.rs           -- unified error type with status codes
  handlers/
    auth.rs           -- register, login
    links.rs          -- create, list, delete short links
    redirect.rs       -- GET /:code -> 302 redirect + click log
    stats.rs          -- per-link analytics
    pages.rs          -- serve HTML templates
  middleware/
    auth.rs           -- JWT creation, verification, AuthUser extractor
    rate_limit.rs     -- in-memory rate limiter
  models/
    user.rs           -- user queries
    link.rs           -- link queries
    click.rs          -- click queries and stats aggregation
  utils/
    hash.rs           -- bcrypt password hashing
    shortcode.rs      -- nanoid-based short code generation
templates/            -- askama HTML templates
migrations/           -- SQL schema
```

## Tech stack

| Layer | Choice | Why |
|-------|--------|-----|
| Language | Rust | Performance, type safety, learning goal |
| Web framework | axum | Async, tower-based, good ergonomics with extractors |
| Async runtime | tokio | Industry standard for async Rust |
| Database | PostgreSQL + sqlx | Compile-time checked queries, async, no ORM overhead |
| Auth | jsonwebtoken + bcrypt | Stateless JWT auth, proven password hashing |
| Templates | Askama | Compile-time HTML, type-safe, no runtime template errors |
| Short codes | nanoid | URL-safe, collision-resistant, configurable length |
| Styling | Tailwind CSS | Utility-first, no build step needed via CDN |
| Containerization | Docker | Multi-stage build for small final image |

## API

```
POST   /api/auth/register    -- create account
POST   /api/auth/login       -- get JWT token
POST   /api/links            -- create short link (auth required)
GET    /api/links             -- list your links (auth required)
DELETE /api/links/:id         -- delete a link (auth required)
GET    /api/links/:id/stats   -- click analytics (auth required)
GET    /:code                 -- redirect to original URL
```

## Run locally

```bash
cp .env.example .env
docker compose up --build
```

Open `http://localhost:3000`.

## Run without Docker

Requires Rust and a running PostgreSQL instance.

```bash
cp .env.example .env
# edit .env with your database credentials
cargo run
```

## Deploy

Tested with Railway. Set these environment variables:

```
DATABASE_URL=<postgres connection string>
JWT_SECRET=<random 64-char hex string>
HOST=0.0.0.0
PORT=<provided by platform>
BASE_URL=https://your-domain.com
```

## What I learned

**Rust ownership in async contexts.** Passing database pools across async boundaries and into `tokio::spawn` requires understanding `Clone`, `Arc`, and move semantics. The redirect handler clones the pool before spawning the click-tracking task because the spawned future must own its data.

**Axum extractors are powerful.** Building a custom `FromRequestParts` implementation for `AuthUser` lets you add authentication to any handler by just adding a parameter. The framework handles the rest. This pattern replaces middleware-based auth in other frameworks.

**`include_str!` is a compile-time decision.** I initially embedded the migration SQL at compile time. This caused a subtle bug where Docker layer caching served a stale binary with outdated SQL. Switching to runtime file reads (and eventually inline statements) fixed it. The lesson: be aware of what happens at compile time vs. runtime, especially in cached build environments.

**Idempotent migrations matter.** `CREATE TABLE` fails if the table exists. `CREATE TABLE IF NOT EXISTS` does not. In production, your app will restart. Your migrations must handle that gracefully. Running each statement individually (not as a batch) also avoids transactional rollback issues in PostgreSQL.

**Frontend scope conflicts in templates.** Template inheritance merges script blocks into one page. Two `const token = ...` declarations in the same global scope cause a `SyntaxError` that kills everything after it. Wrapping base template scripts in IIFEs or using shared helper functions avoids this.

**Docker layer caching is a feature and a trap.** The multi-stage Dockerfile caches dependency compilation in an early layer so only source changes trigger a rebuild. But `COPY . .` captures everything including migration files, and `touch src/main.rs` does not invalidate `include_str!` references to other files. Understanding the cache invalidation chain matters.

**Mobile-first is not optional.** Tables that work on desktop overflow on mobile. The solution is not `overflow-x-auto` (which hides data). It is rendering a completely different layout per breakpoint: table on desktop, card list on mobile. Tailwind's `hidden sm:table` and `sm:hidden` make this clean.

## License

MIT
