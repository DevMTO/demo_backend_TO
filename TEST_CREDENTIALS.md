# Test Credentials

## API Base URL
```
http://localhost:8080/api/v1
```

## Login Endpoint
```
POST /api/v1/auth/login
Content-Type: application/json
```

## Users

| User        | Identifier    | Password    | Role              |
|-------------|---------------|-------------|-------------------|
| Super Admin | superadmin    | admin123    | SuperAdmin        |
| Admin       | admin         | admin123    | Admin             |
| Agencia     | agencia       | agencia123  | Agencia           |
| Agencia Cont| agenciacont   | cont1234    | AgenciaContabilidad|

## PowerShell Login Example (Cookie Session)

```powershell
$base = "http://localhost:8080/api/v1"

# Login admin
$body = @{ identifier = "admin"; password = "admin123" } | ConvertTo-Json
Invoke-RestMethod -Uri "$base/auth/login" -Method POST -Body $body -ContentType "application/json" -SessionVariable adminSession

# Use session in subsequent requests
Invoke-RestMethod -Uri "$base/files" -WebSession $adminSession
```

## Login All Users

```powershell
$base = "http://localhost:8080/api/v1"

# SuperAdmin
$r = Invoke-RestMethod -Uri "$base/auth/login" -Method POST -Body (@{identifier="superadmin";password="admin123"}|ConvertTo-Json) -ContentType "application/json" -SessionVariable superSession

# Admin
$r = Invoke-RestMethod -Uri "$base/auth/login" -Method POST -Body (@{identifier="admin";password="admin123"}|ConvertTo-Json) -ContentType "application/json" -SessionVariable adminSession

# Agencia
$r = Invoke-RestMethod -Uri "$base/auth/login" -Method POST -Body (@{identifier="agencia";password="agencia123"}|ConvertTo-Json) -ContentType "application/json" -SessionVariable agenciaSession

# AgenciaContabilidad
$r = Invoke-RestMethod -Uri "$base/auth/login" -Method POST -Body (@{identifier="agenciacont";password="cont1234"}|ConvertTo-Json) -ContentType "application/json" -SessionVariable contSession
```
