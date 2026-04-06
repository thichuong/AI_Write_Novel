# Python Best Practices for AI Novel Novelist

## Code Style
- Use `snake_case` for variables and functions.
- Use `PascalCase` for classes.
- Follow PEP 8 guidelines.

## FastAPI & Async
- Always use `async def` for route handlers.
- Use `StreamingResponse` for long-running AI generations to provide better UX.
- Keep dependency injection simple and clean.

## Database Consistency
- Use `database.py` utilities for all DB connections.
- Ensure connections are closed properly after use.
- Use parameterized queries to prevent SQL injection.

## AI Integration
- Centralize all AI logic in `agents.py` through the `AIWriter` class.
- Use environment variables for API keys and sensitive configuration.

## Error Handling
- Use structured error responses from FastAPI (HTTPExceptions).
- Log significant errors to console for easier debugging.
