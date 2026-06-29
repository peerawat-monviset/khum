const basketInput = document.getElementById('basket');
const distanceInput = document.getElementById('distance');

const elements = {
    grab: {
        opt: document.getElementById('opt-grab'),
        final: document.getElementById('price-grab-final'),
        original: document.getElementById('price-grab-original'),
    },
    lineman: {
        opt: document.getElementById('opt-lineman'),
        final: document.getElementById('price-lineman-final'),
        original: document.getElementById('price-lineman-original'),
    },
    robinhood: {
        opt: document.getElementById('opt-robinhood'),
        final: document.getElementById('price-robinhood-final'),
        original: document.getElementById('price-robinhood-original'),
    },
    shopee: {
        opt: document.getElementById('opt-shopee'),
        final: document.getElementById('price-shopee-final'),
        original: document.getElementById('price-shopee-original'),
    }
};

let debounceTimeout = null;

function debounceUpdate() {
    clearTimeout(debounceTimeout);
    debounceTimeout = setTimeout(updateComparison, 150);
}

async function updateComparison() {
    const basket = parseFloat(basketInput.value) || 0;
    const distance = parseFloat(distanceInput.value) || 0;

    try {
        const response = await fetch(`/api/calculate?basket=${basket}&distance=${distance}`);
        if (!response.ok) throw new Error('API failed');
        const data = await response.json();
        
        data.providers.forEach(p => {
            const el = elements[p.key];
            if (el) {
                el.final.textContent = `${p.final.toFixed(1)} THB`;
                el.original.textContent = `${p.original.toFixed(1)} THB`;
                
                if (data.best === p.key) {
                    el.opt.classList.add('best-deal');
                } else {
                    el.opt.classList.remove('best-deal');
                }
            }
        });
    } catch (err) {
        console.error(err);
    }
}

const memoryEl = document.getElementById('server-memory');
const cpuEl = document.getElementById('server-cpu');

async function updateMetrics() {
    try {
        const response = await fetch('/api/metrics');
        if (response.ok) {
            const data = await response.json();
            memoryEl.textContent = `${data.mem_current_mb.toFixed(2)} MB / ${data.mem_limit_mb.toFixed(0)} MB`;
            cpuEl.textContent = `${data.cpu_percent.toFixed(1)}%`;
        }
    } catch (err) {
        console.error("Failed to fetch server metrics:", err);
    }
}

const sysinfoEl = document.getElementById('sysinfo');

async function loadSysInfo() {
    try {
        const response = await fetch('/api/sysinfo');
        if (response.ok) {
            const data = await response.json();
            sysinfoEl.innerHTML = `
                Host: ${data.hostname} (${data.hypervisor}) | OS: ${data.os}<br>
                Kernel: ${data.kernel} | Uptime: ${data.uptime}<br>
                CPU: ${data.cpu_model} (${data.cpu_cores} Cores @ ${data.cpu_speed})<br>
                Instruction Flags: <code style="color: var(--accent-primary);">${data.cpu_flags}</code><br>
                Cache: L1-D ${data.l1_data} \| L1-I ${data.l1_inst} \| L2 ${data.l2} \| L3 ${data.l3} \| Line: ${data.cache_line}B<br>
                Page Size: ${data.page_size}B \| NUMA Nodes: ${data.numa_nodes} \| Host RAM: ${data.mem_total}<br>
                Load Avg: ${data.loadavg}
            `;
        }
    } catch (err) {
        console.error("Failed to load VM info:", err);
    }
}

basketInput.addEventListener('input', debounceUpdate);
distanceInput.addEventListener('input', debounceUpdate);

// Initial load
updateComparison();
updateMetrics();
loadSysInfo();

// Refresh server metrics every 5 seconds
setInterval(updateMetrics, 5000);
