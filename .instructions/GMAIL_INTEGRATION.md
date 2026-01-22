# Gmail Integration Instructions for Linux Container

## Overview

This instruction provides comprehensive guidance for an agent running inside a Linux container to connect to Gmail, authenticate headlessly, and perform read and search operations on emails. All authentication flows are container-friendly with no GUI requirements.

## When to Use

Use this instruction when you need to:
- Read emails from a Gmail account within a containerized environment
- Search for specific emails based on criteria (sender, subject, date, labels, etc.)
- Access email metadata (headers, timestamps, recipients)
- Parse email content (body text, HTML, attachments)
- Monitor inbox for new messages
- Filter and organize emails programmatically
- Run automated email processing tasks in Docker/LXC/Podman containers

## Quick Start for Container Agents

**Simplest method (IMAP with App Password):**

1. Generate Gmail App Password at https://myaccount.google.com/apppasswords (requires 2FA)
2. Run container with environment variables:

```bash
docker run -e GMAIL_USER="you@gmail.com" \
           -e GMAIL_APP_PASSWORD="xxxx xxxx xxxx xxxx" \
           your-agent-image
```

3. Use `connect_imap_container()` function from IMAP section - works immediately with no setup

**For Gmail API (better features, requires one-time setup):**

1. Create OAuth token on host machine (one-time, requires browser)
2. Copy `token.pickle` to container via volume mount
3. Use `authenticate_gmail_container()` function - refreshes automatically

See complete examples below for copy-paste ready code.

## Container Environment Setup

### Prerequisites

**System Requirements:**
- Linux container (Docker, LXC, Podman, etc.)
- Python 3.8+ or Node.js 16+
- Internet connectivity from container
- Mounted volume for credential persistence (recommended)

**Container Base Images:**
```dockerfile
# Python-based (Recommended)
FROM python:3.11-slim

# Node.js-based
FROM node:18-slim

# Minimal Alpine (Smallest)
FROM alpine:3.18
RUN apk add --no-cache python3 py3-pip

# Ubuntu-based (Most compatible)
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y python3 python3-pip
```

## Authentication Methods for Containers

### Method 1: Service Account (Recommended for Containers)

**Best for:** Production container deployments, automated systems, no user interaction

**Pros:**
- Fully headless (no browser required)
- No manual authorization flow
- Perfect for containers
- Long-lived credentials
- No refresh token expiration

**Cons:**
- Requires Google Workspace (not available for personal Gmail)
- Requires domain-wide delegation setup
- More complex initial setup

**Setup Steps:**
1. Create a Google Cloud Project
2. Enable Gmail API
3. Create Service Account credentials
4. Download JSON key file
5. Enable Domain-Wide Delegation (for Workspace accounts)
6. Grant service account access to user's mailbox

**Note:** Service accounts only work with Google Workspace accounts, not personal Gmail accounts. For personal Gmail in containers, use Method 2 or Method 3.

### Method 2: OAuth 2.0 with Pre-Generated Token (Recommended for Personal Gmail)

**Best for:** Personal Gmail accounts in containers, agents with pre-authorized access

**Pros:**
- Works with personal Gmail accounts
- Secure token-based authentication
- No passwords stored
- Container-friendly once token is generated

**Cons:**
- Requires one-time manual authorization (outside container)
- Token must be provided to container
- Refresh token may expire (rare, but possible)

**Container Setup:**

**Step 1: Generate Token (Run Once on Host Machine)**
```python
# run_on_host.py - Run this on a machine with a browser
from google_auth_oauthlib.flow import InstalledAppFlow
import pickle

SCOPES = ['https://www.googleapis.com/auth/gmail.readonly']

flow = InstalledAppFlow.from_client_secrets_file(
    'credentials.json', SCOPES)

# This opens a browser for authorization
creds = flow.run_local_server(port=0)

# Save token for container use
with open('token.pickle', 'wb') as token:
    pickle.dump(creds, token)

print("Token saved to token.pickle - copy this file to your container")
```

**Step 2: Use Token in Container**
```dockerfile
# Dockerfile
FROM python:3.11-slim

WORKDIR /app

# Install dependencies
RUN pip install --no-cache-dir google-auth-httplib2 google-auth-oauthlib google-api-python-client

# Copy pre-generated token and credentials
COPY credentials.json /app/credentials.json
COPY token.pickle /app/token.pickle

COPY gmail_agent.py /app/

CMD ["python", "gmail_agent.py"]
```

**Required Scopes:**
```
https://www.googleapis.com/auth/gmail.readonly  # Read-only access
https://www.googleapis.com/auth/gmail.modify    # Read and modify (delete, mark read)
https://www.googleapis.com/auth/gmail.compose   # Send emails
```

### Method 3: App Passwords + IMAP (Simplest for Containers)

**Best for:** Quick setup, simple read-only access, personal Gmail

**Pros:**
- Easiest container setup
- No OAuth complexity
- Works immediately
- No token management
- Standard IMAP protocol

**Cons:**
- Requires 2FA enabled on Gmail
- Less secure (password-based)
- Limited to IMAP capabilities
- Gmail may restrict access

**Setup Steps:**
1. Enable 2-Factor Authentication on Gmail account
2. Go to Google Account > Security > App Passwords
3. Generate a 16-character app password
4. Store password in environment variable or secrets file
5. Use with IMAP in container

**Container Setup:**
```dockerfile
FROM python:3.11-slim

WORKDIR /app

# No special dependencies needed - uses Python's built-in imaplib
COPY gmail_imap_agent.py /app/

# Pass credentials via environment variables
ENV GMAIL_USER="your-email@gmail.com"
ENV GMAIL_APP_PASSWORD="your-app-password"

CMD ["python", "gmail_imap_agent.py"]
```

**Run Container:**
```bash
docker run -e GMAIL_USER="user@gmail.com" -e GMAIL_APP_PASSWORD="xxxx xxxx xxxx xxxx" gmail-agent
```

## Connection Methods

### Option A: Gmail API (Recommended)

**Advantages:**
- Official Google API
- Better rate limits
- Structured responses (JSON)
- Advanced search capabilities
- Batch operations support

**Python Implementation (Container-Friendly):**
```python
from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials
from google.oauth2 import service_account
from googleapiclient.discovery import build
from googleapiclient.errors import HttpError
import os
import pickle
import json

SCOPES = ['https://www.googleapis.com/auth/gmail.readonly']

def authenticate_gmail_container():
    """
    Authenticate in container using pre-generated token
    This function works headlessly without requiring browser access
    """
    # Try environment variable for token path
    token_path = os.getenv('GMAIL_TOKEN_PATH', '/app/token.pickle')
    creds_path = os.getenv('GMAIL_CREDS_PATH', '/app/credentials.json')

    creds = None

    # Load existing token
    if os.path.exists(token_path):
        with open(token_path, 'rb') as token:
            creds = pickle.load(token)

    # Refresh if expired (no browser needed - uses refresh token)
    if creds and creds.expired and creds.refresh_token:
        print("Refreshing expired token...")
        creds.refresh(Request())

        # Save refreshed token
        with open(token_path, 'wb') as token:
            pickle.dump(creds, token)
        print("Token refreshed successfully")

    elif not creds or not creds.valid:
        raise Exception(
            "No valid credentials found. "
            "Generate token on host machine first using run_on_host.py, "
            "then copy token.pickle to container."
        )

    service = build('gmail', 'v1', credentials=creds)
    return service

def authenticate_gmail_service_account(user_email):
    """
    Authenticate using service account (Google Workspace only)
    Fully headless - perfect for containers

    Args:
        user_email: Email address to impersonate
    """
    service_account_file = os.getenv(
        'GMAIL_SERVICE_ACCOUNT_FILE',
        '/app/service-account.json'
    )

    if not os.path.exists(service_account_file):
        raise Exception(f"Service account file not found: {service_account_file}")

    credentials = service_account.Credentials.from_service_account_file(
        service_account_file,
        scopes=SCOPES
    )

    # Impersonate user (requires domain-wide delegation)
    delegated_credentials = credentials.with_subject(user_email)

    service = build('gmail', 'v1', credentials=delegated_credentials)
    return service
```

**Node.js Implementation:**
```javascript
const fs = require('fs').promises;
const path = require('path');
const {authenticate} = require('@google-cloud/local-auth');
const {google} = require('googleapis');

const SCOPES = ['https://www.googleapis.com/auth/gmail.readonly'];
const TOKEN_PATH = path.join(process.cwd(), 'token.json');
const CREDENTIALS_PATH = path.join(process.cwd(), 'credentials.json');

async function loadSavedCredentialsIfExist() {
  try {
    const content = await fs.readFile(TOKEN_PATH);
    const credentials = JSON.parse(content);
    return google.auth.fromJSON(credentials);
  } catch (err) {
    return null;
  }
}

async function saveCredentials(client) {
  const content = await fs.readFile(CREDENTIALS_PATH);
  const keys = JSON.parse(content);
  const key = keys.installed || keys.web;
  const payload = JSON.stringify({
    type: 'authorized_user',
    client_id: key.client_id,
    client_secret: key.client_secret,
    refresh_token: client.credentials.refresh_token,
  });
  await fs.writeFile(TOKEN_PATH, payload);
}

async function authorize() {
  let client = await loadSavedCredentialsIfExist();
  if (client) {
    return client;
  }
  client = await authenticate({
    scopes: SCOPES,
    keyfilePath: CREDENTIALS_PATH,
  });
  if (client.credentials) {
    await saveCredentials(client);
  }
  return client;
}

async function getGmailService() {
  const auth = await authorize();
  return google.gmail({version: 'v1', auth});
}
```

### Option B: IMAP Protocol

**Advantages:**
- Standard email protocol
- Works with app passwords
- Simpler for basic operations
- No API setup required

**Python Implementation (Container-Friendly IMAP):**
```python
import imaplib
import email
import os
from email.header import decode_header

def connect_imap_container():
    """
    Connect to Gmail via IMAP using environment variables
    Perfect for containers - no file dependencies
    """
    email_address = os.getenv('GMAIL_USER')
    app_password = os.getenv('GMAIL_APP_PASSWORD')

    if not email_address or not app_password:
        raise Exception(
            "Missing credentials. Set GMAIL_USER and GMAIL_APP_PASSWORD "
            "environment variables"
        )

    # Remove spaces from app password (Gmail format: "xxxx xxxx xxxx xxxx")
    app_password = app_password.replace(' ', '')

    imap = imaplib.IMAP4_SSL("imap.gmail.com", 993)
    imap.login(email_address, app_password)

    return imap

def select_mailbox(imap, mailbox="INBOX"):
    """Select a mailbox/folder"""
    status, messages = imap.select(mailbox)
    return int(messages[0])

def disconnect_imap(imap):
    """Close IMAP connection"""
    try:
        imap.close()
        imap.logout()
    except:
        pass  # Already disconnected
```

## Reading Emails

### Using Gmail API

**List Messages:**
```python
def list_messages(service, user_id='me', query='', max_results=10):
    """
    List messages matching query

    Args:
        service: Gmail API service instance
        user_id: User's email (default 'me' for authenticated user)
        query: Gmail search query (e.g., 'from:sender@example.com')
        max_results: Maximum number of messages to return

    Returns:
        List of message objects
    """
    try:
        response = service.users().messages().list(
            userId=user_id,
            q=query,
            maxResults=max_results
        ).execute()

        messages = response.get('messages', [])
        return messages
    except HttpError as error:
        print(f'An error occurred: {error}')
        return []
```

**Get Message Details:**
```python
def get_message(service, user_id='me', msg_id=''):
    """
    Get full message details

    Args:
        service: Gmail API service instance
        user_id: User's email
        msg_id: Message ID

    Returns:
        Message object with full details
    """
    try:
        message = service.users().messages().get(
            userId=user_id,
            id=msg_id,
            format='full'  # Options: 'minimal', 'full', 'raw', 'metadata'
        ).execute()
        return message
    except HttpError as error:
        print(f'An error occurred: {error}')
        return None
```

**Parse Message Content:**
```python
import base64

def parse_message(message):
    """
    Parse message and extract relevant information

    Returns:
        Dictionary with parsed email data
    """
    headers = message['payload']['headers']

    # Extract headers
    subject = next((h['value'] for h in headers if h['name'] == 'Subject'), 'No Subject')
    sender = next((h['value'] for h in headers if h['name'] == 'From'), 'Unknown')
    date = next((h['value'] for h in headers if h['name'] == 'Date'), 'Unknown')
    to = next((h['value'] for h in headers if h['name'] == 'To'), 'Unknown')

    # Extract body
    body = ''
    if 'parts' in message['payload']:
        for part in message['payload']['parts']:
            if part['mimeType'] == 'text/plain':
                data = part['body'].get('data', '')
                body = base64.urlsafe_b64decode(data).decode('utf-8')
                break
            elif part['mimeType'] == 'text/html':
                data = part['body'].get('data', '')
                body = base64.urlsafe_b64decode(data).decode('utf-8')
    else:
        data = message['payload']['body'].get('data', '')
        if data:
            body = base64.urlsafe_b64decode(data).decode('utf-8')

    return {
        'id': message['id'],
        'thread_id': message['threadId'],
        'subject': subject,
        'from': sender,
        'to': to,
        'date': date,
        'body': body,
        'snippet': message.get('snippet', ''),
        'labels': message.get('labelIds', [])
    }
```

### Using IMAP

**Fetch Messages:**
```python
def fetch_emails_imap(imap, num_messages=10):
    """
    Fetch most recent emails via IMAP

    Args:
        imap: IMAP connection object
        num_messages: Number of recent emails to fetch

    Returns:
        List of parsed email objects
    """
    # Get total message count
    status, messages = imap.select("INBOX")
    message_count = int(messages[0])

    emails = []

    # Fetch most recent messages
    for i in range(message_count, max(message_count - num_messages, 0), -1):
        status, msg_data = imap.fetch(str(i), "(RFC822)")

        for response_part in msg_data:
            if isinstance(response_part, tuple):
                msg = email.message_from_bytes(response_part[1])

                # Decode subject
                subject, encoding = decode_header(msg["Subject"])[0]
                if isinstance(subject, bytes):
                    subject = subject.decode(encoding if encoding else "utf-8")

                # Get sender
                from_ = msg.get("From")

                # Get body
                body = ""
                if msg.is_multipart():
                    for part in msg.walk():
                        content_type = part.get_content_type()
                        if content_type == "text/plain":
                            body = part.get_payload(decode=True).decode()
                            break
                else:
                    body = msg.get_payload(decode=True).decode()

                emails.append({
                    'subject': subject,
                    'from': from_,
                    'body': body,
                    'date': msg.get("Date")
                })

    return emails
```

## Searching Emails

### Gmail API Search Queries

**Query Syntax:**
```
from:user@example.com          # From specific sender
to:recipient@example.com       # To specific recipient
subject:keyword                # Subject contains keyword
has:attachment                 # Has attachments
filename:pdf                   # Attachment with specific extension
label:important                # Has specific label
is:unread                      # Unread messages
is:read                        # Read messages
is:starred                     # Starred messages
after:2024/01/01               # After specific date
before:2024/12/31              # Before specific date
newer_than:7d                  # Last 7 days
older_than:1m                  # Older than 1 month
larger:5M                      # Larger than 5MB
smaller:1M                     # Smaller than 1MB
in:inbox                       # In inbox
in:trash                       # In trash
in:spam                        # In spam
```

**Combine Queries:**
```python
# Multiple conditions (AND)
query = "from:boss@company.com subject:urgent is:unread"

# OR conditions
query = "from:sender1@example.com OR from:sender2@example.com"

# Exclude terms
query = "subject:meeting -subject:cancelled"

# Complex query
query = "(from:hr@company.com OR from:payroll@company.com) subject:salary after:2024/01/01"
```

**Search Implementation:**
```python
def search_emails(service, query, max_results=100):
    """
    Search emails with advanced query

    Args:
        service: Gmail API service
        query: Search query string
        max_results: Maximum results to return

    Returns:
        List of matching message objects with full details
    """
    try:
        # Get message IDs matching query
        results = service.users().messages().list(
            userId='me',
            q=query,
            maxResults=max_results
        ).execute()

        messages = results.get('messages', [])

        # Fetch full details for each message
        detailed_messages = []
        for msg in messages:
            full_msg = get_message(service, msg_id=msg['id'])
            parsed = parse_message(full_msg)
            detailed_messages.append(parsed)

        return detailed_messages

    except HttpError as error:
        print(f'An error occurred: {error}')
        return []
```

### IMAP Search

**IMAP Search Criteria:**
```python
def search_imap(imap, criteria):
    """
    Search emails via IMAP

    Args:
        imap: IMAP connection
        criteria: Search criteria string

    IMAP Search Examples:
        'ALL'                          # All messages
        'UNSEEN'                       # Unread messages
        'SEEN'                         # Read messages
        'FLAGGED'                      # Starred messages
        'FROM "sender@example.com"'    # From sender
        'TO "recipient@example.com"'   # To recipient
        'SUBJECT "keyword"'            # Subject contains
        'BODY "text"'                  # Body contains
        'SINCE "01-Jan-2024"'          # Since date
        'BEFORE "31-Dec-2024"'         # Before date
        'LARGER 1000000'               # Larger than bytes
        'SMALLER 1000000'              # Smaller than bytes

    Combine with:
        '(FROM "sender" SUBJECT "urgent")'  # AND
        '(OR FROM "a@ex.com" FROM "b@ex.com")'  # OR
        'NOT SEEN'                      # NOT

    Returns:
        List of message IDs
    """
    status, message_ids = imap.search(None, criteria)

    if status == 'OK':
        return message_ids[0].split()
    return []
```

## Advanced Operations

### Batch Operations

**Fetch Multiple Messages:**
```python
def batch_get_messages(service, message_ids):
    """
    Efficiently fetch multiple messages

    Args:
        service: Gmail API service
        message_ids: List of message IDs

    Returns:
        List of parsed message objects
    """
    from googleapiclient.http import BatchHttpRequest

    messages = []

    def callback(request_id, response, exception):
        if exception:
            print(f'Error: {exception}')
        else:
            messages.append(parse_message(response))

    batch = service.new_batch_http_request(callback=callback)

    for msg_id in message_ids:
        batch.add(service.users().messages().get(userId='me', id=msg_id))

    batch.execute()
    return messages
```

### Handle Attachments

**Download Attachments (Gmail API):**
```python
def get_attachments(service, msg_id, download_path='./attachments'):
    """
    Download all attachments from a message

    Args:
        service: Gmail API service
        msg_id: Message ID
        download_path: Directory to save attachments

    Returns:
        List of saved file paths
    """
    import os

    os.makedirs(download_path, exist_ok=True)

    message = service.users().messages().get(
        userId='me',
        id=msg_id
    ).execute()

    saved_files = []

    for part in message['payload'].get('parts', []):
        if part['filename']:
            attachment_id = part['body'].get('attachmentId')

            if attachment_id:
                attachment = service.users().messages().attachments().get(
                    userId='me',
                    messageId=msg_id,
                    id=attachment_id
                ).execute()

                data = attachment['data']
                file_data = base64.urlsafe_b64decode(data)

                file_path = os.path.join(download_path, part['filename'])

                with open(file_path, 'wb') as f:
                    f.write(file_data)

                saved_files.append(file_path)

    return saved_files
```

### Pagination

**Handle Large Result Sets:**
```python
def get_all_messages(service, query='', max_total=1000):
    """
    Get all messages matching query with pagination

    Args:
        service: Gmail API service
        query: Search query
        max_total: Maximum total messages to retrieve

    Returns:
        List of all matching message IDs
    """
    all_messages = []
    page_token = None

    while len(all_messages) < max_total:
        try:
            if page_token:
                response = service.users().messages().list(
                    userId='me',
                    q=query,
                    pageToken=page_token,
                    maxResults=500
                ).execute()
            else:
                response = service.users().messages().list(
                    userId='me',
                    q=query,
                    maxResults=500
                ).execute()

            messages = response.get('messages', [])
            all_messages.extend(messages)

            page_token = response.get('nextPageToken')

            if not page_token:
                break

        except HttpError as error:
            print(f'An error occurred: {error}')
            break

    return all_messages[:max_total]
```

## Rate Limits and Quotas

### Gmail API Limits
- **Quota**: 1 billion quota units per day (free tier)
- **Rate Limit**: 250 quota units per second per user
- **Batch Size**: Maximum 100 requests per batch

**Quota Costs:**
- `messages.list()`: 5 units
- `messages.get()`: 5 units (minimal), 10 units (full)
- `messages.send()`: 100 units

**Best Practices:**
```python
import time

def rate_limited_request(service, func, *args, **kwargs):
    """Execute request with rate limiting"""
    max_retries = 3
    retry_delay = 1

    for attempt in range(max_retries):
        try:
            return func(*args, **kwargs)
        except HttpError as error:
            if error.resp.status == 429:  # Rate limit exceeded
                wait_time = retry_delay * (2 ** attempt)
                print(f'Rate limit hit, waiting {wait_time}s...')
                time.sleep(wait_time)
            else:
                raise

    raise Exception('Max retries exceeded')
```

### IMAP Limitations
- **Connection Limit**: 15 simultaneous connections per account
- **Download Limit**: Varies by account (typically 2.5GB/day)
- **Request Rate**: Gmail may throttle aggressive polling

## Error Handling

**Comprehensive Error Handling:**
```python
from googleapiclient.errors import HttpError
import socket
import ssl

def safe_gmail_operation(service, operation_func, *args, **kwargs):
    """
    Safely execute Gmail operation with error handling

    Args:
        service: Gmail API service
        operation_func: Function to execute
        *args, **kwargs: Arguments for the function

    Returns:
        Result of operation or None on error
    """
    try:
        return operation_func(*args, **kwargs)

    except HttpError as error:
        error_code = error.resp.status

        if error_code == 400:
            print('Bad Request: Invalid parameters')
        elif error_code == 401:
            print('Unauthorized: Authentication failed')
        elif error_code == 403:
            print('Forbidden: Insufficient permissions')
        elif error_code == 404:
            print('Not Found: Resource does not exist')
        elif error_code == 429:
            print('Rate Limit Exceeded: Too many requests')
        elif error_code >= 500:
            print('Server Error: Gmail service issue')
        else:
            print(f'HTTP Error {error_code}: {error}')

        return None

    except socket.error as error:
        print(f'Network Error: {error}')
        return None

    except ssl.SSLError as error:
        print(f'SSL Error: {error}')
        return None

    except Exception as error:
        print(f'Unexpected Error: {error}')
        return None
```

## Security Best Practices

1. **Credential Storage:**
   - Never hardcode credentials in source code
   - Use environment variables or secure credential managers
   - Encrypt stored tokens
   - Use `.gitignore` for `credentials.json` and `token.pickle`

2. **Scope Minimization:**
   - Request only necessary scopes
   - Use `gmail.readonly` when write access isn't needed
   - Avoid `gmail.full` scope unless absolutely required

3. **Token Management:**
   - Implement token refresh logic
   - Handle expired tokens gracefully
   - Revoke tokens when no longer needed

4. **Connection Security:**
   - Always use SSL/TLS connections
   - Verify SSL certificates
   - Avoid public WiFi for sensitive operations

5. **Data Privacy:**
   - Don't log full email content
   - Sanitize output/error messages
   - Follow data retention policies
   - Implement proper access controls

## Complete Container-Ready Examples

### Example 1: Gmail API with Pre-Generated Token

**File: `gmail_agent.py`**
```python
#!/usr/bin/env python3
"""
Container-ready Gmail agent using pre-generated OAuth token
No browser required - fully headless
"""

from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials
from googleapiclient.discovery import build
from googleapiclient.errors import HttpError
import os
import pickle
import base64
from datetime import datetime, timedelta

SCOPES = ['https://www.googleapis.com/auth/gmail.readonly']

def authenticate_gmail_container():
    """Authenticate using pre-generated token"""
    token_path = os.getenv('GMAIL_TOKEN_PATH', '/app/token.pickle')

    if not os.path.exists(token_path):
        raise Exception(
            f"Token not found at {token_path}. "
            "Generate token.pickle on host first."
        )

    with open(token_path, 'rb') as token:
        creds = pickle.load(token)

    # Auto-refresh if expired
    if creds and creds.expired and creds.refresh_token:
        print("Refreshing token...")
        creds.refresh(Request())
        with open(token_path, 'wb') as token:
            pickle.dump(creds, token)

    if not creds or not creds.valid:
        raise Exception("Invalid credentials")

    return build('gmail', 'v1', credentials=creds)

def parse_message(message):
    """Parse Gmail API message"""
    headers = message['payload']['headers']

    subject = next((h['value'] for h in headers if h['name'] == 'Subject'), 'No Subject')
    sender = next((h['value'] for h in headers if h['name'] == 'From'), 'Unknown')
    date = next((h['value'] for h in headers if h['name'] == 'Date'), 'Unknown')

    # Extract body
    body = ''
    if 'parts' in message['payload']:
        for part in message['payload']['parts']:
            if part['mimeType'] == 'text/plain':
                data = part['body'].get('data', '')
                if data:
                    body = base64.urlsafe_b64decode(data).decode('utf-8')
                    break
    else:
        data = message['payload']['body'].get('data', '')
        if data:
            body = base64.urlsafe_b64decode(data).decode('utf-8')

    return {
        'id': message['id'],
        'subject': subject,
        'from': sender,
        'date': date,
        'body': body,
        'snippet': message.get('snippet', '')
    }

def search_emails(service, query, max_results=100):
    """Search emails with query"""
    try:
        results = service.users().messages().list(
            userId='me',
            q=query,
            maxResults=max_results
        ).execute()

        message_ids = results.get('messages', [])

        # Fetch full details
        messages = []
        for msg in message_ids:
            full_msg = service.users().messages().get(
                userId='me',
                id=msg['id'],
                format='full'
            ).execute()
            messages.append(parse_message(full_msg))

        return messages

    except HttpError as error:
        print(f'Error: {error}')
        return []

def main():
    print("Gmail Agent starting in container...")

    # Authenticate
    service = authenticate_gmail_container()
    print("Authentication successful!")

    # Example 1: Get unread emails from last 7 days
    seven_days_ago = (datetime.now() - timedelta(days=7)).strftime('%Y/%m/%d')
    query = f'is:unread after:{seven_days_ago}'

    print(f'\nSearching: {query}')
    messages = search_emails(service, query, max_results=50)
    print(f'Found {len(messages)} unread messages')

    for i, msg in enumerate(messages[:5], 1):  # Show first 5
        print(f"\n--- Email {i} ---")
        print(f"From: {msg['from']}")
        print(f"Subject: {msg['subject']}")
        print(f"Preview: {msg['snippet'][:100]}...")

    # Example 2: Search for specific sender
    sender_email = os.getenv('SEARCH_SENDER', 'important@company.com')
    sender_query = f'from:{sender_email}'
    results = search_emails(service, sender_query, max_results=10)
    print(f"\n\nFound {len(results)} emails from {sender_email}")

    # Example 3: Get emails with attachments
    attachment_query = 'has:attachment newer_than:7d'
    pdfs = search_emails(service, attachment_query, max_results=20)
    print(f"Found {len(pdfs)} emails with attachments in last 7 days")

if __name__ == '__main__':
    try:
        main()
    except Exception as e:
        print(f"Error: {e}")
        exit(1)
```

**File: `Dockerfile`**
```dockerfile
FROM python:3.11-slim

WORKDIR /app

# Install dependencies
RUN pip install --no-cache-dir \
    google-auth-httplib2 \
    google-auth-oauthlib \
    google-api-python-client

# Copy application
COPY gmail_agent.py /app/

# Credentials and token will be mounted as volumes
# docker run -v $(pwd)/credentials.json:/app/credentials.json \
#            -v $(pwd)/token.pickle:/app/token.pickle \
#            gmail-agent

CMD ["python", "-u", "gmail_agent.py"]
```

**Build and Run:**
```bash
# Build container
docker build -t gmail-agent .

# Run with mounted credentials
docker run -v $(pwd)/token.pickle:/app/token.pickle gmail-agent

# Or with environment variable
docker run -e GMAIL_TOKEN_PATH=/credentials/token.pickle \
           -v $(pwd)/token.pickle:/credentials/token.pickle \
           gmail-agent
```

### Example 2: IMAP (Simplest Container Setup)

**File: `gmail_imap_agent.py`**
```python
#!/usr/bin/env python3
"""
Container-ready Gmail agent using IMAP + App Password
Simplest setup - only needs environment variables
"""

import imaplib
import email
import os
from email.header import decode_header

def connect_gmail():
    """Connect using environment variables"""
    user = os.getenv('GMAIL_USER')
    password = os.getenv('GMAIL_APP_PASSWORD')

    if not user or not password:
        raise Exception("Set GMAIL_USER and GMAIL_APP_PASSWORD environment variables")

    # Remove spaces from app password
    password = password.replace(' ', '')

    imap = imaplib.IMAP4_SSL("imap.gmail.com")
    imap.login(user, password)
    print(f"Connected as {user}")

    return imap

def fetch_unread_emails(imap, max_count=10):
    """Fetch unread emails"""
    imap.select("INBOX")

    # Search for unread messages
    status, messages = imap.search(None, 'UNSEEN')

    if status != 'OK':
        print("No messages found")
        return []

    message_ids = messages[0].split()
    emails = []

    # Get most recent unread emails
    for msg_id in message_ids[-max_count:]:
        status, msg_data = imap.fetch(msg_id, '(RFC822)')

        for response_part in msg_data:
            if isinstance(response_part, tuple):
                msg = email.message_from_bytes(response_part[1])

                # Decode subject
                subject, encoding = decode_header(msg["Subject"])[0]
                if isinstance(subject, bytes):
                    subject = subject.decode(encoding if encoding else "utf-8")

                # Get body
                body = ""
                if msg.is_multipart():
                    for part in msg.walk():
                        if part.get_content_type() == "text/plain":
                            body = part.get_payload(decode=True).decode()
                            break
                else:
                    body = msg.get_payload(decode=True).decode()

                emails.append({
                    'from': msg.get("From"),
                    'subject': subject,
                    'date': msg.get("Date"),
                    'body': body[:500]  # First 500 chars
                })

    return emails

def search_emails_by_sender(imap, sender_email):
    """Search emails from specific sender"""
    imap.select("INBOX")

    # Search for emails from sender
    status, messages = imap.search(None, f'FROM "{sender_email}"')

    if status != 'OK':
        return []

    message_ids = messages[0].split()
    return message_ids

def main():
    print("Gmail IMAP Agent starting...")

    # Connect
    imap = connect_gmail()

    # Get unread emails
    print("\n=== Unread Emails ===")
    unread = fetch_unread_emails(imap, max_count=5)
    print(f"Found {len(unread)} unread emails")

    for i, email_msg in enumerate(unread, 1):
        print(f"\n--- Email {i} ---")
        print(f"From: {email_msg['from']}")
        print(f"Subject: {email_msg['subject']}")
        print(f"Preview: {email_msg['body'][:100]}...")

    # Search for specific sender
    search_sender = os.getenv('SEARCH_SENDER', 'noreply@github.com')
    print(f"\n=== Emails from {search_sender} ===")
    results = search_emails_by_sender(imap, search_sender)
    print(f"Found {len(results)} emails")

    # Cleanup
    imap.logout()
    print("\nDone!")

if __name__ == '__main__':
    try:
        main()
    except Exception as e:
        print(f"Error: {e}")
        exit(1)
```

**File: `Dockerfile`**
```dockerfile
FROM python:3.11-slim

WORKDIR /app

# No extra dependencies needed - uses built-in imaplib

COPY gmail_imap_agent.py /app/

# Credentials passed via environment variables
CMD ["python", "-u", "gmail_imap_agent.py"]
```

**Build and Run:**
```bash
# Build
docker build -t gmail-imap-agent .

# Run with credentials
docker run \
  -e GMAIL_USER="your-email@gmail.com" \
  -e GMAIL_APP_PASSWORD="xxxx xxxx xxxx xxxx" \
  -e SEARCH_SENDER="important@company.com" \
  gmail-imap-agent

# Or using env file
echo "GMAIL_USER=your-email@gmail.com" > .env
echo "GMAIL_APP_PASSWORD=xxxx xxxx xxxx xxxx" >> .env

docker run --env-file .env gmail-imap-agent
```

### Example 3: Docker Compose Setup

**File: `docker-compose.yml`**
```yaml
version: '3.8'

services:
  gmail-agent:
    build: .
    environment:
      - GMAIL_USER=${GMAIL_USER}
      - GMAIL_APP_PASSWORD=${GMAIL_APP_PASSWORD}
      - SEARCH_SENDER=${SEARCH_SENDER:-noreply@github.com}
    volumes:
      # For OAuth token persistence
      - ./token.pickle:/app/token.pickle:ro
      - ./output:/app/output
    restart: unless-stopped
    # Run every hour
    command: >
      sh -c "while true; do
        python -u gmail_agent.py
        sleep 3600
      done"
```

**Run with Docker Compose:**
```bash
# Create .env file
cat > .env << EOF
GMAIL_USER=your-email@gmail.com
GMAIL_APP_PASSWORD=xxxx xxxx xxxx xxxx
SEARCH_SENDER=important@company.com
EOF

# Start service
docker-compose up -d

# View logs
docker-compose logs -f gmail-agent

# Stop service
docker-compose down
```

## Credential Configuration for Containers

### Option 1: Environment Variables (Recommended)

**For IMAP/App Password:**
```bash
# Set in container environment or .env file
GMAIL_USER=your-email@gmail.com
GMAIL_APP_PASSWORD=xxxx xxxx xxxx xxxx
SEARCH_SENDER=optional-filter@example.com  # Optional
```

**For OAuth with Token:**
```bash
# Token file path
GMAIL_TOKEN_PATH=/app/token.pickle
GMAIL_CREDS_PATH=/app/credentials.json

# Mount files as volumes:
# -v $(pwd)/token.pickle:/app/token.pickle:ro
```

### Option 2: Mounted Secrets (Docker Swarm/Kubernetes)

**Docker Secrets:**
```bash
# Create secrets
echo "your-email@gmail.com" | docker secret create gmail_user -
echo "xxxx xxxx xxxx xxxx" | docker secret create gmail_password -

# Use in service
docker service create \
  --secret gmail_user \
  --secret gmail_password \
  --env GMAIL_USER_FILE=/run/secrets/gmail_user \
  --env GMAIL_PASSWORD_FILE=/run/secrets/gmail_password \
  gmail-agent
```

**Kubernetes Secrets:**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: gmail-credentials
type: Opaque
stringData:
  user: your-email@gmail.com
  password: xxxx xxxx xxxx xxxx
---
apiVersion: v1
kind: Pod
metadata:
  name: gmail-agent
spec:
  containers:
  - name: agent
    image: gmail-agent:latest
    env:
    - name: GMAIL_USER
      valueFrom:
        secretKeyRef:
          name: gmail-credentials
          key: user
    - name: GMAIL_APP_PASSWORD
      valueFrom:
        secretKeyRef:
          name: gmail-credentials
          key: password
```

### Option 3: Configuration File Mount

**File: `config.json`**
```json
{
  "gmail_address": "your-email@gmail.com",
  "auth_method": "oauth2",
  "token_path": "/app/token.pickle",
  "credentials_path": "/app/credentials.json",
  "scopes": ["https://www.googleapis.com/auth/gmail.readonly"]
}
```

**Or for IMAP:**
```json
{
  "gmail_address": "your-email@gmail.com",
  "auth_method": "app_password",
  "app_password": "xxxx xxxx xxxx xxxx"
}
```

**Mount in container:**
```bash
docker run -v $(pwd)/config.json:/app/config.json:ro gmail-agent
```

### Credential Placeholder

**Actual credentials to be provided:**

- [ ] Gmail account email address: `________________@gmail.com`
- [ ] Authentication method: `☐ OAuth2 ☐ App Password`
- [ ] If OAuth: `credentials.json` file (obtain from Google Cloud Console)
- [ ] If OAuth: Pre-generated `token.pickle` (run auth script on host first)
- [ ] If App Password: 16-character password: `____ ____ ____ ____`
- [ ] Required scopes (if OAuth): `☐ readonly ☐ modify ☐ compose`
- [ ] Container runtime: `☐ Docker ☐ Podman ☐ LXC ☐ Other: _______`

## Container Dependencies

### Python Requirements

**For Gmail API (OAuth):**
```dockerfile
# In Dockerfile
RUN pip install --no-cache-dir \
    google-auth-httplib2 \
    google-auth-oauthlib \
    google-api-python-client
```

**Or requirements.txt:**
```
google-auth-httplib2==0.1.1
google-auth-oauthlib==1.1.0
google-api-python-client==2.108.0
```

**For IMAP (No dependencies):**
```python
# Uses Python built-in libraries only:
import imaplib  # Built-in
import email    # Built-in
```

### Node.js Requirements

**For Gmail API:**
```dockerfile
# In Dockerfile
RUN npm install googleapis @google-cloud/local-auth
```

**package.json:**
```json
{
  "dependencies": {
    "googleapis": "^128.0.0",
    "@google-cloud/local-auth": "^3.0.0"
  }
}
```

### Container Image Sizes

**Minimal Python IMAP:** ~50MB (python:3.11-slim + code)
**Python Gmail API:** ~200MB (python:3.11-slim + Google libs + code)
**Node.js Gmail API:** ~250MB (node:18-slim + googleapis + code)
**Alpine Python IMAP:** ~30MB (alpine + python3 + code)

## Container Troubleshooting

### Common Container Issues

1. **"Token not found" error in container:**
   ```bash
   # Verify token is mounted correctly
   docker run --rm -v $(pwd)/token.pickle:/app/token.pickle \
     gmail-agent ls -la /app/

   # Check token path matches code
   docker run --rm -e GMAIL_TOKEN_PATH=/app/token.pickle \
     -v $(pwd)/token.pickle:/app/token.pickle \
     gmail-agent python -c "import os; print(os.getenv('GMAIL_TOKEN_PATH'))"
   ```

2. **"Connection refused" or network errors:**
   ```bash
   # Test internet connectivity from container
   docker run --rm gmail-agent ping -c 3 imap.gmail.com

   # Check DNS resolution
   docker run --rm gmail-agent nslookup imap.gmail.com

   # Try with host network mode
   docker run --network host gmail-agent
   ```

3. **"Invalid credentials" error (IMAP):**
   ```bash
   # Verify credentials are passed correctly
   docker run --rm \
     -e GMAIL_USER="test@gmail.com" \
     -e GMAIL_APP_PASSWORD="test1234test5678" \
     gmail-agent python -c "import os; print(os.getenv('GMAIL_USER'), len(os.getenv('GMAIL_APP_PASSWORD', '')))"

   # Check for spaces in password
   # App password format: "xxxx xxxx xxxx xxxx" (16 chars with spaces)
   # Code should remove spaces: password.replace(' ', '')
   ```

4. **"Token expired" in container:**
   ```bash
   # Token refresh requires write access
   # Mount with read-write (remove :ro)
   docker run -v $(pwd)/token.pickle:/app/token.pickle gmail-agent

   # Or mount entire directory
   docker run -v $(pwd):/credentials \
     -e GMAIL_TOKEN_PATH=/credentials/token.pickle \
     gmail-agent
   ```

5. **"Permission denied" writing token:**
   ```bash
   # Fix file permissions before mounting
   chmod 644 token.pickle

   # Or run container as specific user
   docker run --user $(id -u):$(id -g) \
     -v $(pwd)/token.pickle:/app/token.pickle \
     gmail-agent
   ```

6. **Environment variables not working:**
   ```bash
   # Debug: Print all env vars in container
   docker run --rm -e GMAIL_USER="test" gmail-agent env | grep GMAIL

   # Use --env-file for multiple variables
   docker run --env-file .env gmail-agent

   # Verify .env file format (no quotes, no spaces around =)
   cat .env
   GMAIL_USER=user@gmail.com
   GMAIL_APP_PASSWORD=xxxx xxxx xxxx xxxx
   ```

7. **SSL/TLS certificate errors:**
   ```bash
   # Install ca-certificates in container
   # Add to Dockerfile:
   RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

   # Or for Alpine:
   RUN apk add --no-cache ca-certificates
   ```

8. **IMAP "authentication failed" despite correct credentials:**
   ```bash
   # Check if Gmail blocks sign-in from container
   # - Enable "Less secure app access" (if available)
   # - Use App Password (requires 2FA)
   # - Check Google Account security alerts

   # Test IMAP manually
   docker run --rm -it python:3.11-slim python3
   >>> import imaplib
   >>> imap = imaplib.IMAP4_SSL("imap.gmail.com")
   >>> imap.login("user@gmail.com", "app-password")
   ```

### Gmail-Specific Container Issues

1. **"Access blocked" for new container IPs:**
   - Gmail may block unfamiliar IPs/locations
   - Check Gmail security alerts email
   - Verify sign-in attempt in Google Account activity
   - Use OAuth instead of app passwords for better reliability

2. **Rate limiting in containers:**
   ```python
   # Implement exponential backoff
   import time
   from googleapiclient.errors import HttpError

   def retry_with_backoff(func, max_retries=3):
       for attempt in range(max_retries):
           try:
               return func()
           except HttpError as e:
               if e.resp.status == 429 and attempt < max_retries - 1:
                   wait = (2 ** attempt) * 1
                   time.sleep(wait)
               else:
                   raise
   ```

3. **Container restarts lose connection:**
   ```python
   # Implement reconnection logic
   def get_imap_connection(max_retries=3):
       for attempt in range(max_retries):
           try:
               imap = connect_imap_container()
               imap.select("INBOX")  # Test connection
               return imap
           except Exception as e:
               if attempt < max_retries - 1:
                   time.sleep(5)
               else:
                   raise
   ```

### Debugging Tips

**Enable verbose logging:**
```python
import logging
logging.basicConfig(level=logging.DEBUG)

# For Gmail API
import googleapiclient.discovery
googleapiclient.discovery.logger.setLevel(logging.DEBUG)

# For IMAP
import imaplib
imaplib.Debug = 4  # Maximum verbosity
```

**Test authentication separately:**
```bash
# Create test script
cat > test_auth.py << 'EOF'
import os
import sys

auth_method = os.getenv('AUTH_METHOD', 'imap')

if auth_method == 'imap':
    import imaplib
    user = os.getenv('GMAIL_USER')
    password = os.getenv('GMAIL_APP_PASSWORD', '').replace(' ', '')
    print(f"Testing IMAP login for {user}...")
    imap = imaplib.IMAP4_SSL("imap.gmail.com")
    imap.login(user, password)
    print("✓ IMAP authentication successful!")
    imap.logout()
else:
    import pickle
    token_path = os.getenv('GMAIL_TOKEN_PATH', '/app/token.pickle')
    print(f"Testing OAuth token at {token_path}...")
    with open(token_path, 'rb') as f:
        creds = pickle.load(f)
    print(f"✓ Token loaded. Valid: {creds.valid}, Expired: {creds.expired}")
    if creds.expired:
        print("⚠ Token is expired but has refresh token" if creds.refresh_token else "✗ Token expired and no refresh token")

print("\n✓ All authentication checks passed!")
EOF

# Run test
docker run --rm \
  -v $(pwd)/test_auth.py:/app/test_auth.py \
  -e GMAIL_USER="user@gmail.com" \
  -e GMAIL_APP_PASSWORD="xxxx xxxx xxxx xxxx" \
  gmail-agent python /app/test_auth.py
```

## Additional Resources

- Gmail API Documentation: https://developers.google.com/gmail/api
- Gmail API Python Quickstart: https://developers.google.com/gmail/api/quickstart/python
- Gmail Search Operators: https://support.google.com/mail/answer/7190
- IMAP Protocol: https://www.rfc-editor.org/rfc/rfc3501
- Google Cloud Console: https://console.cloud.google.com

---

**Note:** This instruction file provides technical implementation details. Actual credentials, account details, and authorization tokens should be provided separately and stored securely.
