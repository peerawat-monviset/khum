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

async function updateMetrics() {
    try {
        const response = await fetch('/api/metrics');
        if (response.ok) {
            const data = await response.json();
            memoryEl.textContent = `${data.mem_current_mb.toFixed(2)} MB / ${data.mem_limit_mb.toFixed(0)} MB`;
        }
    } catch (err) {
        console.error("Failed to fetch server metrics:", err);
    }
}

basketInput.addEventListener('input', debounceUpdate);
distanceInput.addEventListener('input', debounceUpdate);

// Initial load
updateComparison();
updateMetrics();

// Refresh server metrics every 5 seconds
setInterval(updateMetrics, 5000);
