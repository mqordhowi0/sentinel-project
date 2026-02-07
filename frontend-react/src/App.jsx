import { useState, useEffect } from 'react';
import axios from 'axios';

function App() {
  const [url, setUrl] = useState('');
  const [history, setHistory] = useState([]); // Untuk nampung daftar scan
  const [loading, setLoading] = useState(false);

  // Fungsi Kirim Link ke Go API
  const handleSubmit = async (e) => {
    e.preventDefault();
    setLoading(true);
    try {
      const res = await axios.post('/api/scan', { url });
      const newScan = { id: res.data.id, url: url, status: 'pending' };
      setHistory([newScan, ...history]); // Masukkan ke list paling atas
      setUrl('');
    } catch (err) {
      alert("Pastikan Backend Go sudah jalan di port 8080!");
    } finally {
      setLoading(false);
    }
  };

  // Logic Polling: Cek semua yang statusnya 'pending' setiap 3 detik
  useEffect(() => {
    const interval = setInterval(async () => {
      const pendingItems = history.filter(item => item.status === 'pending');
      
      if (pendingItems.length > 0) {
        const updatedHistory = await Promise.all(history.map(async (item) => {
          if (item.status === 'pending') {
            try {
              const res = await axios.get(`/api/check?id=${item.id}`);
              return { ...item, status: res.data.status };
            } catch (e) { return item; }
          }
          return item;
        }));
        setHistory(updatedHistory);
      }
    }, 3000);

    return () => clearInterval(interval);
  }, [history]);

  return (
    <div className="min-h-screen bg-slate-900 text-slate-100 p-8 font-sans">
      <div className="max-w-2xl mx-auto">
        {/* Header */}
        <header className="text-center mb-12">
          <h1 className="text-4xl font-black tracking-tighter bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent">
            SENTINEL PRO
          </h1>
          <p className="text-slate-400 mt-2">Real-time Link Threat Intelligence</p>
        </header>

        {/* Input Card */}
        <div className="bg-slate-800 border border-slate-700 p-6 rounded-2xl shadow-xl mb-12">
          <form onSubmit={handleSubmit} className="flex gap-3">
          <input
            type="url"
            className="flex-1 bg-transparent p-4 outline-none font-mono text-sm"
            placeholder="ENTER URL TO SCAN..."
            value={url} // Ini harus nyambung ke state
            onChange={(e) => setUrl(e.target.value)} // Ini harus meng-update state
            required
          />
            <button
              disabled={loading}
              className="bg-cyan-600 hover:bg-cyan-500 px-8 rounded-xl font-bold transition-colors disabled:opacity-50"
            >
              {loading ? 'Processing...' : 'Analyze'}
            </button>
          </form>
        </div>

        {/* History List */}
        <div className="space-y-4">
          <h2 className="text-xl font-bold text-slate-400 mb-6">Recent Scans</h2>
          {history.length === 0 && <p className="text-center text-slate-600 italic">No links scanned yet.</p>}
          
          {history.map((item) => (
            <div key={item.id} className="bg-slate-800/50 border border-slate-700 p-5 rounded-xl flex justify-between items-center animate-in slide-in-from-top-4">
              <div className="overflow-hidden pr-4">
                <p className="text-xs text-slate-500 mb-1">SCAN ID #{item.id}</p>
                <p className="font-medium truncate text-slate-300">{item.url}</p>
              </div>
              <StatusBadge status={item.status} />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function StatusBadge({ status }) {
  const config = {
    pending: "bg-amber-500/10 text-amber-500 border-amber-500/20 animate-pulse",
    safe: "bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
    dangerous: "bg-red-500/10 text-red-500 border-red-500/20",
  };

  return (
    <span className={`px-4 py-1.5 rounded-full text-xs font-black border uppercase tracking-wider ${config[status]}`}>
      {status === 'pending' ? '🔍 Checking' : status}
    </span>
  );
}

export default App;