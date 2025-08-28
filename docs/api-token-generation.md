# PAS API Token Generation Guide

This guide explains how to generate proper v2 API tokens for PAS server integration.

## Overview

PAS supports two types of API authentication:
1. **Web UI Tokens** - Generated from web interface, limited to UI operations
2. **V2 API Tokens** - OAuth2 client credentials, full API access

For integration testing and SDK development, you need **V2 API Tokens**.

## V2 API Token Generation

### Prerequisites
- PAS server access with valid username/password
- Network access to the PAS server API endpoints
- `curl` or similar HTTP client

### Step 1: Register API Client

Register a new API client to get `client_id` and `client_secret`:

```bash
curl -k -X POST "https://your-pas-server.com/rss-servlet/api/v2/auth/register" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=YOUR_USERNAME" \
  -d "password=YOUR_PASSWORD" \
  -d "type=API_KEY" \
  -d "description=Integration Test Client"
```

**Client Types**:
- `API_KEY` - General API access
- `SCM_API` - Secure Connection Manager API
- `INTEGRATION` - Third-party integrations
- `MOBILE` - Mobile applications
- `WEB_UI` - Web interface

**Response**:
```json
{
  "client_id": "abc123-def456-ghi789",
  "client_secret": "xyz789-uvw456-rst123"
}
```

### Step 2: Get Access Token

Use the client credentials to get an access token:

```bash
curl -k -X POST "https://your-pas-server.com/rss-servlet/api/v2/auth/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials" \
  -d "client_id=abc123-def456-ghi789" \
  -d "client_secret=xyz789-uvw456-rst123"
```

**Response**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

### Step 3: Use Access Token

Use the access token in API requests:

```bash
curl -k "https://your-pas-server.com/rss-servlet/api/v2/scmkey?access_token=YOUR_ACCESS_TOKEN"
```

## Integration Test Configuration

### Environment Variables

Add to your workspace `.envrc`:

```bash
# PAS Integration Test Configuration
export PAS_SERVER_URL="https://your-pas-server.com"
export PAS_API_TOKEN="your-v2-access-token"
export PAS_IGNORE_SSL_ERRORS="true"  # For dev/test environments
```

### CLI Usage

```bash
# Automatic (using environment variables)
./build/integration-test-rssconnect

# CLI override
./build/integration-test-rssconnect \
  --server-url "https://server.com" \
  --api-token "your-token"
```

## Token Management

### Token Expiration
- V2 API tokens expire after 24 hours (86400 seconds)
- Regenerate tokens as needed
- Consider implementing token refresh in automation

### Security Best Practices
- Store `client_secret` securely
- Rotate client credentials periodically
- Use network restrictions when possible
- Never commit tokens to version control

### Network Restrictions (Optional)

Restrict API client to specific networks:

```bash
curl -k -X POST "https://your-pas-server.com/rss-servlet/api/v2/auth/register" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=YOUR_USERNAME" \
  -d "password=YOUR_PASSWORD" \
  -d "type=API_KEY" \
  -d "description=Integration Test Client" \
  -d "networkRestrictions=10.0.0.0/8,192.168.0.0/16"
```

## Troubleshooting

### Common Issues

**401 Unauthorized**
- Check username/password for client registration
- Verify client_id/client_secret for token generation
- Ensure token hasn't expired

**403 Forbidden**
- Check user permissions
- Verify network restrictions
- Ensure client type has required permissions

**415 Unsupported Media Type**
- Use `application/x-www-form-urlencoded` content type
- Don't use JSON for auth endpoints

### Debugging

Enable verbose curl output:
```bash
curl -k -v -X POST "https://server.com/rss-servlet/api/v2/auth/register" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=user&password=pass&type=API_KEY&description=test"
```

## Example: Complete Flow

```bash
#!/bin/bash
set -e

SERVER="https://dh-vpam-01.securelink.eng"
USERNAME="your-username"
PASSWORD="your-password"

echo "1. Registering API client..."
REGISTER_RESPONSE=$(curl -k -s -X POST "$SERVER/rss-servlet/api/v2/auth/register" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=$USERNAME&password=$PASSWORD&type=API_KEY&description=Integration Test")

CLIENT_ID=$(echo "$REGISTER_RESPONSE" | jq -r '.client_id')
CLIENT_SECRET=$(echo "$REGISTER_RESPONSE" | jq -r '.client_secret')

echo "2. Getting access token..."
TOKEN_RESPONSE=$(curl -k -s -X POST "$SERVER/rss-servlet/api/v2/auth/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials&client_id=$CLIENT_ID&client_secret=$CLIENT_SECRET")

ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')

echo "3. Testing API access..."
curl -k "$SERVER/rss-servlet/api/v2/scmkey?access_token=$ACCESS_TOKEN"

echo "Access token: $ACCESS_TOKEN"
```

## References

- OAuth2 Client Credentials Flow: RFC 6749 Section 4.4
- PAS API Documentation: [Internal Wiki/Confluence]
- Integration Test Documentation: `librssconnect/README.md`
