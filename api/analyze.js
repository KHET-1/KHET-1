// StarFire — PCAP Analysis API Route
// Accepts uploaded pcap files, runs through ingest pipeline
// Returns report metadata + links

import { createHash } from 'crypto';
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'fs';

const STORE_PATH = '/tmp/lattice_results.json';

function loadResults() {
    try {
        if (existsSync(STORE_PATH)) return JSON.parse(readFileSync(STORE_PATH, 'utf8'));
    } catch {}
    return [];
}

function saveResults(results) {
    if (results.length > 100) results = results.slice(-100);
    writeFileSync(STORE_PATH, JSON.stringify(results));
}

export const config = {
    api: { bodyParser: false }
};

export default async function handler(req, res) {
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

    if (req.method === 'OPTIONS') return res.status(200).end();

    if (req.method !== 'POST') {
        return res.status(405).json({ error: 'POST only' });
    }

    const chunks = [];
    let totalSize = 0;

    for await (const chunk of req) {
        chunks.push(chunk);
        totalSize += chunk.length;
        if (totalSize > 50 * 1024 * 1024) {
            return res.status(413).json({ error: 'File too large (50MB max). Use CLI: ./starfire analyze file.pcap' });
        }
    }

    const buffer = Buffer.concat(chunks);
    const sha256 = createHash('sha256').update(buffer).digest('hex');
    const reportId = `rpt-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;

    const entry = {
        id: reportId,
        type: 'pcap_upload',
        received_at: new Date().toISOString(),
        size_bytes: buffer.length,
        sha256: sha256,
        status: 'received',
        note: 'Full StarFire analysis requires CLI toolkit (serverless cannot run tshark). File hashed and sealed.'
    };

    const results = loadResults();
    results.push(entry);
    saveResults(results);

    return res.status(200).json({
        success: true,
        reportId: reportId,
        reportUrl: `/agents/results.html`,
        merkleLeaf: sha256,
        verdict: 'RECEIVED — Run ./starfire analyze locally for full council analysis',
        summary: `File sealed (${(buffer.length/1024/1024).toFixed(2)} MB, SHA-256: ${sha256.slice(0,16)}...). Full analysis requires local toolkit with tshark.`,
        note: 'Serverless environment cannot run pyshark/tshark. Download toolkit for full offline analysis.'
    });
}
