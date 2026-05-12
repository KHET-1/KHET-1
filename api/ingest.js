// LATTICE TEAM — KHET-1 — Ingest API
// Receives capture results from phone/desktop agents
// Stores in-memory for display on Lattice IDE (Vercel serverless)

const results = [];

export default async function handler(req, res) {
    // CORS for airgap/cross-origin
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, X-Device-ID, X-Capture-Time');

    if (req.method === 'OPTIONS') {
        return res.status(200).end();
    }

    if (req.method === 'POST') {
        const deviceId = req.headers['x-device-id'] || 'unknown';
        const captureTime = req.headers['x-capture-time'] || new Date().toISOString();
        const contentType = req.headers['content-type'] || '';

        let body = '';
        const chunks = [];
        let totalSize = 0;

        // Collect body
        for await (const chunk of req) {
            chunks.push(chunk);
            totalSize += chunk.length;
            if (totalSize > 50 * 1024 * 1024) {
                return res.status(413).json({ error: 'Payload too large (50MB max)' });
            }
        }

        const buffer = Buffer.concat(chunks);

        // If JSON, parse and store directly
        let payload;
        if (contentType.includes('application/json')) {
            try {
                payload = JSON.parse(buffer.toString());
            } catch {
                payload = { raw: buffer.toString().slice(0, 10000) };
            }
        } else {
            // Binary (tarball) — store metadata, base64 first 1KB for preview
            payload = {
                type: 'binary_capture',
                size_bytes: buffer.length,
                preview_b64: buffer.slice(0, 1024).toString('base64'),
                sha256: require('crypto').createHash('sha256').update(buffer).digest('hex')
            };
        }

        const entry = {
            id: `capture-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
            device_id: deviceId,
            capture_time: captureTime,
            received_at: new Date().toISOString(),
            content_type: contentType,
            size_bytes: buffer.length,
            payload
        };

        // Store (in-memory — resets on cold start; for persistence, use KV/Blob)
        // For now, write to /tmp which persists within a single function instance
        const fs = require('fs');
        const resultsFile = '/tmp/lattice_results.json';
        let existing = [];
        try {
            existing = JSON.parse(fs.readFileSync(resultsFile, 'utf8'));
        } catch {}
        existing.push(entry);
        // Keep last 100 results
        if (existing.length > 100) existing = existing.slice(-100);
        fs.writeFileSync(resultsFile, JSON.stringify(existing, null, 2));

        return res.status(201).json({
            status: 'accepted',
            id: entry.id,
            device_id: deviceId,
            size_bytes: buffer.length,
            message: `Capture ingested. View at /api/results`
        });
    }

    if (req.method === 'GET') {
        // Return all stored results
        const fs = require('fs');
        const resultsFile = '/tmp/lattice_results.json';
        let existing = [];
        try {
            existing = JSON.parse(fs.readFileSync(resultsFile, 'utf8'));
        } catch {}

        return res.status(200).json({
            count: existing.length,
            results: existing
        });
    }

    return res.status(405).json({ error: 'Method not allowed' });
}
