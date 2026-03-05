#!/usr/bin/env node

const { put } = require('@vercel/blob');
const fs = require('fs');
const path = require('path');

async function uploadFile(filePath, blobName) {
  try {
    const file = fs.readFileSync(filePath);
    
    console.log(`📤 Uploading ${blobName}...`);
    
    const blob = await put(blobName, file, {
      access: 'public',
      addRandomSuffix: false, // This prevents the hash from being added!
      token: process.env.VERCEL_BLOB_TOKEN,
      cacheControlMaxAge: 0,
    });

    console.log(`✅ Upload successful!`);
    console.log(`📦 Download URL: ${blob.url}`);
    
    return blob;
  } catch (error) {
    console.error(`❌ Upload failed:`, error.message);
    process.exit(1);
  }
}

// Get arguments
const filePath = process.argv[2];
const blobName = process.argv[3];

if (!filePath || !blobName) {
  console.error('Usage: node upload-to-vercel.js <file-path> <blob-name>');
  process.exit(1);
}

if (!process.env.VERCEL_BLOB_TOKEN) {
  console.error('❌ VERCEL_BLOB_TOKEN environment variable is not set');
  process.exit(1);
}

uploadFile(filePath, blobName);
