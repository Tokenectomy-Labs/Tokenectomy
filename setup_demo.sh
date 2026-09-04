#!/bin/bash
set -e

rm -rf ~/.cache/tokenectomy/responses

cat << 'PY' > server.py
import os

def calculate_metrics():
    # Sensitive credential inside local application logic
    db_conn = "postgres://admin:sk_live_9942a8b3c109f4e@db.internal:5432/analytics"
    total_tokens = 50000
    active_users = 0
    per_user = total_tokens / active_users
    return per_user

def start_server():
    print("🚀 [Production API] Starting services on port 8080...")
    calculate_metrics()

if __name__ == "__main__":
    start_server()
PY

cat << 'LOG' > error.log
Traceback (most recent call last):
  File "/usr/lib/python3.13/site-packages/uvicorn/protocols/http/httptools_impl.py", line 426, in run_asgi
    result = await app(scope, receive, send)
  File "/usr/lib/python3.13/site-packages/uvicorn/middleware/proxy_headers.py", line 84, in __call__
    return await self.app(scope, receive, send)
  File "/usr/lib/python3.13/site-packages/fastapi/applications.py", line 1054, in __call__
    await super().__call__(scope, receive, send)
  File "/usr/lib/python3.13/site-packages/starlette/applications.py", line 123, in __call__
    await self.middleware_stack(scope, receive, send)
  File "/usr/lib/python3.13/site-packages/starlette/middleware/errors.py", line 186, in __call__
    raise exc
  File "/usr/lib/python3.13/site-packages/starlette/middleware/exceptions.py", line 79, in __call__
    raise exc
  File "/usr/lib/python3.13/site-packages/starlette/middleware/base.py", line 108, in __call__
    response = await self.dispatch_func(request, call_next)
  File "/usr/lib/python3.13/site-packages/starlette/middleware/cors.py", line 85, in __call__
    await self.app(scope, receive, send)
  File "/usr/lib/python3.13/site-packages/starlette/middleware/gzip.py", line 24, in __call__
    await self.app(scope, receive, send)
  File "/usr/lib/python3.13/site-packages/starlette/routing.py", line 715, in __call__
    await route.handle(scope, receive, send)
  File "/usr/lib/python3.13/site-packages/starlette/routing.py", line 275, in handle
    await self.app(scope, receive, send)
  File "/usr/lib/python3.13/site-packages/anyio/_backends/_asyncio.py", line 805, in run
    result = context.run(func, *args)
  File "/usr/lib/python3.13/site-packages/pydantic/v1/main.py", line 341, in __init__
    values, fields_set, validation_error = validate_model(__pydantic_self__.__class__, data)
  File "/usr/lib/python3.13/site-packages/sqlalchemy/orm/session.py", line 1928, in execute
    return self._execute_internal(
  File "/usr/lib/python3.13/site-packages/sqlalchemy/engine/base.py", line 1408, in execute
    return meth(self, multiparams, params, _EMPTY_EXECUTION_OPTS)
  File "./server.py", line 12, in start_server
    calculate_metrics()
  File "./server.py", line 7, in calculate_metrics
    per_user = total_tokens / active_users
ZeroDivisionError: division by zero
LOG

chmod +x setup_demo.sh
