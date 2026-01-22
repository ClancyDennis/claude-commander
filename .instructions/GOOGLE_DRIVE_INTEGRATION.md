# Google Drive Integration Guide

This guide covers integrating with Google Drive API for file storage, retrieval, and management operations.

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Prerequisites](#prerequisites)
3. [Configuration](#configuration)
4. [Authentication](#authentication)
5. [File Operations](#file-operations)
6. [Folder Management](#folder-management)
7. [Sharing & Permissions](#sharing--permissions)
8. [Error Handling](#error-handling)

---

## Quick Reference

### Credentials (Fill in your values)

| Item | Value | Description |
|------|-------|-------------|
| Project ID | `YOUR_PROJECT_ID` | Google Cloud project |
| Client ID | `YOUR_CLIENT_ID` | OAuth client ID |
| Client Secret | `YOUR_CLIENT_SECRET` | OAuth client secret |
| Service Account | `YOUR_SERVICE_ACCOUNT@YOUR_PROJECT.iam.gserviceaccount.com` | For server-to-server auth |

### Environment Variables

```bash
# Google Drive Configuration
# Copy to your .env file and fill in your values

GOOGLE_CLOUD_PROJECT=YOUR_PROJECT_ID

# OAuth 2.0 credentials (for user consent flow)
GOOGLE_DRIVE_CLIENT_ID=YOUR_CLIENT_ID
GOOGLE_DRIVE_CLIENT_SECRET=YOUR_CLIENT_SECRET
GOOGLE_DRIVE_REDIRECT_URI=http://localhost:3000/oauth/callback

# Pre-generated refresh token (after initial auth)
GOOGLE_DRIVE_REFRESH_TOKEN=YOUR_REFRESH_TOKEN

# Service account (alternative - for server-to-server)
# GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account-key.json
```

### API Scopes

```bash
# Read-only access
https://www.googleapis.com/auth/drive.readonly

# Full access
https://www.googleapis.com/auth/drive

# File-level access (created by your app only)
https://www.googleapis.com/auth/drive.file

# Metadata only
https://www.googleapis.com/auth/drive.metadata.readonly
```

---

## Prerequisites

### 1. Google Cloud Setup

**Enable Drive API:**
```bash
gcloud services enable drive.googleapis.com
```

**Create OAuth Credentials:**
```bash
# Via Google Cloud Console:
# 1. Go to APIs & Services > Credentials
# 2. Create OAuth 2.0 Client ID
# 3. Application type: Web application (or Desktop)
# 4. Add authorized redirect URIs
# 5. Download JSON and save credentials
```

### 2. Software Requirements

```bash
# Node.js
npm install googleapis google-auth-library

# Python
pip install google-api-python-client google-auth-httplib2 google-auth-oauthlib
```

---

## Configuration

### Load Environment Variables

```typescript
// config.ts
import { config } from 'dotenv';
config();

export const DRIVE_CONFIG = {
  projectId: process.env.GOOGLE_CLOUD_PROJECT!,
  clientId: process.env.GOOGLE_DRIVE_CLIENT_ID!,
  clientSecret: process.env.GOOGLE_DRIVE_CLIENT_SECRET!,
  redirectUri: process.env.GOOGLE_DRIVE_REDIRECT_URI || 'http://localhost:3000/oauth/callback',
  refreshToken: process.env.GOOGLE_DRIVE_REFRESH_TOKEN,
};
```

---

## Authentication

### Method 1: OAuth 2.0 with Refresh Token (Recommended for User Files)

**Step 1: Generate Authorization URL**

```typescript
import { google } from 'googleapis';
import { DRIVE_CONFIG } from './config';

const oauth2Client = new google.auth.OAuth2(
  DRIVE_CONFIG.clientId,
  DRIVE_CONFIG.clientSecret,
  DRIVE_CONFIG.redirectUri
);

function getAuthUrl(): string {
  return oauth2Client.generateAuthUrl({
    access_type: 'offline', // Gets refresh token
    scope: ['https://www.googleapis.com/auth/drive.file'],
    prompt: 'consent', // Force consent to get refresh token
  });
}

console.log('Visit this URL:', getAuthUrl());
```

**Step 2: Exchange Code for Tokens**

```typescript
async function getTokensFromCode(code: string) {
  const { tokens } = await oauth2Client.getToken(code);
  console.log('Refresh Token:', tokens.refresh_token);
  // Save refresh_token to GOOGLE_DRIVE_REFRESH_TOKEN env var
  return tokens;
}
```

**Step 3: Use Refresh Token**

```typescript
async function getAuthenticatedClient() {
  oauth2Client.setCredentials({
    refresh_token: DRIVE_CONFIG.refreshToken,
  });
  return oauth2Client;
}

const drive = google.drive({ version: 'v3', auth: await getAuthenticatedClient() });
```

### Method 2: Service Account (For Server-to-Server)

```typescript
import { google } from 'googleapis';

async function getServiceAccountAuth() {
  const auth = new google.auth.GoogleAuth({
    scopes: ['https://www.googleapis.com/auth/drive'],
  });
  return auth;
}

const auth = await getServiceAccountAuth();
const drive = google.drive({ version: 'v3', auth });
```

### Using Raw fetch (Without SDK)

```typescript
async function getAccessToken(): Promise<string> {
  const response = await fetch('https://oauth2.googleapis.com/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      client_id: DRIVE_CONFIG.clientId,
      client_secret: DRIVE_CONFIG.clientSecret,
      refresh_token: DRIVE_CONFIG.refreshToken!,
      grant_type: 'refresh_token',
    }),
  });

  const data = await response.json();
  return data.access_token;
}
```

---

## File Operations

### List Files

```typescript
async function listFiles(options?: {
  pageSize?: number;
  query?: string;
  folderId?: string;
  orderBy?: string;
}) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  let q = options?.query || '';
  if (options?.folderId) {
    q = `'${options.folderId}' in parents${q ? ' and ' + q : ''}`;
  }

  const response = await drive.files.list({
    pageSize: options?.pageSize || 100,
    q: q || undefined,
    fields: 'nextPageToken, files(id, name, mimeType, size, createdTime, modifiedTime, parents)',
    orderBy: options?.orderBy || 'modifiedTime desc',
  });

  return response.data.files || [];
}

// Examples
// List all files
const allFiles = await listFiles();

// Search by name
const byName = await listFiles({ query: "name contains 'report'" });

// List folder contents
const folderContents = await listFiles({ folderId: 'FOLDER_ID' });

// Only PDFs
const pdfs = await listFiles({ query: "mimeType = 'application/pdf'" });
```

### Upload File

```typescript
import { Readable } from 'stream';

async function uploadFile(
  name: string,
  content: Buffer | string | Readable,
  mimeType: string,
  folderId?: string
) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  const media = {
    mimeType,
    body: content instanceof Buffer
      ? Readable.from(content)
      : typeof content === 'string'
        ? Readable.from(content)
        : content,
  };

  const response = await drive.files.create({
    requestBody: {
      name,
      parents: folderId ? [folderId] : undefined,
    },
    media,
    fields: 'id, name, mimeType, size, webViewLink',
  });

  return response.data;
}

// Example: Upload text file
const textFile = await uploadFile(
  'notes.txt',
  'Hello, World!',
  'text/plain'
);

// Example: Upload to specific folder
const inFolder = await uploadFile(
  'report.pdf',
  pdfBuffer,
  'application/pdf',
  'FOLDER_ID'
);
```

### Download File

```typescript
async function downloadFile(fileId: string): Promise<Buffer> {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  const response = await drive.files.get(
    { fileId, alt: 'media' },
    { responseType: 'arraybuffer' }
  );

  return Buffer.from(response.data as ArrayBuffer);
}

// Download Google Docs as PDF
async function exportGoogleDoc(
  fileId: string,
  mimeType: string = 'application/pdf'
): Promise<Buffer> {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  const response = await drive.files.export(
    { fileId, mimeType },
    { responseType: 'arraybuffer' }
  );

  return Buffer.from(response.data as ArrayBuffer);
}
```

### Get File Metadata

```typescript
async function getFileMetadata(fileId: string) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  const response = await drive.files.get({
    fileId,
    fields: 'id, name, mimeType, size, createdTime, modifiedTime, parents, webViewLink, webContentLink, permissions',
  });

  return response.data;
}
```

### Update File

```typescript
async function updateFile(
  fileId: string,
  content: Buffer | string,
  mimeType: string
) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  const response = await drive.files.update({
    fileId,
    media: {
      mimeType,
      body: Readable.from(content instanceof Buffer ? content : Buffer.from(content)),
    },
    fields: 'id, name, modifiedTime',
  });

  return response.data;
}
```

### Delete File

```typescript
async function deleteFile(fileId: string): Promise<void> {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  await drive.files.delete({ fileId });
}

// Move to trash instead
async function trashFile(fileId: string) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  return drive.files.update({
    fileId,
    requestBody: { trashed: true },
  });
}
```

---

## Folder Management

### Create Folder

```typescript
async function createFolder(name: string, parentId?: string) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  const response = await drive.files.create({
    requestBody: {
      name,
      mimeType: 'application/vnd.google-apps.folder',
      parents: parentId ? [parentId] : undefined,
    },
    fields: 'id, name, webViewLink',
  });

  return response.data;
}
```

### List Folders Only

```typescript
async function listFolders(parentId?: string) {
  return listFiles({
    query: "mimeType = 'application/vnd.google-apps.folder'",
    folderId: parentId,
  });
}
```

### Move File to Folder

```typescript
async function moveFile(fileId: string, newFolderId: string) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  // Get current parents
  const file = await drive.files.get({
    fileId,
    fields: 'parents',
  });

  const previousParents = file.data.parents?.join(',') || '';

  // Move to new folder
  const response = await drive.files.update({
    fileId,
    addParents: newFolderId,
    removeParents: previousParents,
    fields: 'id, name, parents',
  });

  return response.data;
}
```

---

## Sharing & Permissions

### Share File

```typescript
async function shareFile(
  fileId: string,
  email: string,
  role: 'reader' | 'writer' | 'commenter' = 'reader'
) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  const response = await drive.permissions.create({
    fileId,
    requestBody: {
      type: 'user',
      role,
      emailAddress: email,
    },
    sendNotificationEmail: true,
  });

  return response.data;
}

// Share with anyone (link sharing)
async function shareWithLink(
  fileId: string,
  role: 'reader' | 'writer' = 'reader'
) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  await drive.permissions.create({
    fileId,
    requestBody: {
      type: 'anyone',
      role,
    },
  });

  // Get the shareable link
  const file = await drive.files.get({
    fileId,
    fields: 'webViewLink',
  });

  return file.data.webViewLink;
}
```

### List Permissions

```typescript
async function listPermissions(fileId: string) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  const response = await drive.permissions.list({
    fileId,
    fields: 'permissions(id, type, role, emailAddress, displayName)',
  });

  return response.data.permissions || [];
}
```

### Remove Permission

```typescript
async function removePermission(fileId: string, permissionId: string) {
  const auth = await getAuthenticatedClient();
  const drive = google.drive({ version: 'v3', auth });

  await drive.permissions.delete({
    fileId,
    permissionId,
  });
}
```

---

## Common MIME Types

| Google Type | MIME Type | Export As |
|-------------|-----------|-----------|
| Document | `application/vnd.google-apps.document` | PDF, DOCX, TXT |
| Spreadsheet | `application/vnd.google-apps.spreadsheet` | XLSX, CSV, PDF |
| Presentation | `application/vnd.google-apps.presentation` | PPTX, PDF |
| Folder | `application/vnd.google-apps.folder` | - |

| File Type | MIME Type |
|-----------|-----------|
| PDF | `application/pdf` |
| JSON | `application/json` |
| Text | `text/plain` |
| CSV | `text/csv` |
| PNG | `image/png` |
| JPEG | `image/jpeg` |

---

## Error Handling

```typescript
import { GaxiosError } from 'gaxios';

class DriveError extends Error {
  constructor(
    message: string,
    public statusCode: number,
    public errorCode?: string
  ) {
    super(message);
    this.name = 'DriveError';
  }
}

async function handleDriveRequest<T>(
  requestFn: () => Promise<T>
): Promise<T> {
  try {
    return await requestFn();
  } catch (error) {
    if (error instanceof GaxiosError) {
      const status = error.response?.status || 500;
      const message = error.response?.data?.error?.message || error.message;
      const code = error.response?.data?.error?.errors?.[0]?.reason;

      throw new DriveError(message, status, code);
    }
    throw error;
  }
}

// Usage
try {
  const files = await handleDriveRequest(() => listFiles());
} catch (error) {
  if (error instanceof DriveError) {
    if (error.statusCode === 404) {
      console.log('File not found');
    } else if (error.statusCode === 403) {
      console.log('Access denied');
    }
  }
}
```

---

## Best Practices

1. **Use drive.file scope** - Request minimal permissions (only files created by your app)
2. **Implement pagination** - Use `nextPageToken` for large file lists
3. **Handle rate limits** - Implement exponential backoff for 429 errors
4. **Cache access tokens** - Tokens are valid for 1 hour
5. **Use fields parameter** - Only request needed fields to reduce response size
6. **Batch requests** - Use batch API for multiple operations
7. **Store refresh token securely** - Never commit to version control

---

## Related Resources

- [Google Drive API Documentation](https://developers.google.com/drive/api/v3/about-sdk)
- [OAuth 2.0 for Web Apps](https://developers.google.com/identity/protocols/oauth2/web-server)
- [Drive API Reference](https://developers.google.com/drive/api/v3/reference)
