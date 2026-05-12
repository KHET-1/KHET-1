// LATTICE TEAM — Results API
// Returns all ingested capture results for display

export default async function handler(req, res) {
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
    res.setHeader('Cache-Control', 'no-cache, no-store');

    if (req.method === 'OPTIONS') return res.status(200).end();

    const fs = require('fs');
    const resultsFile = '/tmp/lattice_results.json';
    let results = [];
    try {
        results = JSON.parse(fs.readFileSync(resultsFile, 'utf8'));
    } catch {}

    const deviceFilter = req.query.device;
    if (deviceFilter) {
        results = results.filter(r => r.device_id.includes(deviceFilter));
    }

    return res.status(200).json({
        operation: 'LATTICE TEAM — KHET-1',
        count: results.length,
        last_updated: results.length > 0 ? results[results.length - 1].received_at : null,
        results: results.map(r => ({
            id: r.id,
            device_id: r.device_id,
            capture_time: r.capture_time,
            received_at: r.received_at,
            size_bytes: r.size_bytes,
            content_type: r.content_type,
            payload: r.payload
        }))
    });
}
