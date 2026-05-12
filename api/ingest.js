// LATTICE TEAM — KHET-1 — Ingest + Results API
// Uses /tmp for persistence within warm instances

import { createHash } from 'crypto';
import { readFileSync, writeFileSync, existsSync } from 'fs';

const STORE_PATH = '/tmp/lattice_results.json';

function loadResults() {
    try {
        if (existsSync(STORE_PATH)) {
            return JSON.parse(readFileSync(STORE_PATH, 'utf8'));
        }
    } catch {}
    return [];
}

function saveResults(results) {
    if (results.length > 100) results = results.slice(-100);
    writeFileSync(STORE_PATH, JSON.stringify(results));
}

export default async function handler(req, res) {
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, X-Device-ID, X-Capture-Time');

    if (req.method === 'OPTIONS') return res.status(200).end();

    if (req.method === 'GET') {
        const results = loadResults();
        return res.status(200).json({
            operation: 'LATTICE TEAM — KHET-1',
            count: results.length,
            last_updated: results.length > 0 ? results[results.length - 1].received_at : null,
            results
        });
    }

    if (req.method === 'POST') {
        const deviceId = req.headers['x-device-id'] || 'unknown-device';
        const captureTime = req.headers['x-capture-time'] || new Date().toISOString();
        const contentType = req.headers['content-type'] || '';

        const chunks = [];
        let totalSize = 0;
        for await (const chunk of req) {
            chunks.push(chunk);
            totalSize += chunk.length;
            if (totalSize > 50 * 1024 * 1024) {
                return res.status(413).json({ error: 'Payload too large (50MB max)' });
            }
        }
        const buffer = Buffer.concat(chunks);

        let payload;
        if (contentType.includes('application/json')) {
            try { payload = JSON.parse(buffer.toString()); }
            catch { payload = { raw: buffer.toString().slice(0, 10000) }; }
        } else if (contentType.includes('text/plain')) {
            payload = { type: 'text_capture', content: buffer.toString().slice(0, 50000) };
        } else {
            payload = {
                type: 'binary_capture',
                size_bytes: buffer.length,
                sha256: createHash('sha256').update(buffer).digest('hex'),
                preview_hex: buffer.slice(0, 256).toString('hex')
            };
        }

        const entry = {
            id: `cap-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
            device_id: deviceId,
            capture_time: captureTime,
            received_at: new Date().toISOString(),
            content_type: contentType,
            size_bytes: buffer.length,
            payload
        };

        const results = loadResults();
        results.push(entry);
        saveResults(results);

        return res.status(201).json({
            status: 'accepted',
            id: entry.id,
            device_id: deviceId,
            size_bytes: buffer.length,
            total_captures: results.length,
            message: 'Capture ingested. View at /api/ingest (GET) or /agents/results.html'
        });
    }

    return res.status(405).json({ error: 'Method not allowed. Use GET or POST.' });
}
