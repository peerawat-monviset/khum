const basketInput = document.getElementById('basket');
const distanceInput = document.getElementById('distance');
const comparisonList = document.getElementById('comparison-list');

async function updateComparison() {
    const basket = parseFloat(basketInput.value) || 0;
    const distance = parseFloat(distanceInput.value) || 0;

    try {
        const response = await fetch(`/api/calculate?basket=${basket}&distance=${distance}`);
        if (!response.ok) throw new Error('API failed');
        const data = await response.json();
        renderList(data);
    } catch (err) {
        console.error(err);
    }
}

function renderList(data) {
    comparisonList.innerHTML = '';
    data.providers.forEach(provider => {
        const isBest = provider.key === data.best;
        const item = document.createElement('div');
        item.className = `delivery-option ${isBest ? 'best-deal' : ''}`;
        
        item.innerHTML = `
            <div class="provider-info">
                <div class="provider-dot ${provider.key}"></div>
                <span class="provider-name">${provider.name}</span>
            </div>
            <div class="price-details">
                <div class="final-price">${provider.final.toFixed(1)} THB</div>
                <div class="original-price">${provider.original.toFixed(1)} THB</div>
            </div>
        `;
        comparisonList.appendChild(item);
    });
}

basketInput.addEventListener('input', updateComparison);
distanceInput.addEventListener('input', updateComparison);

// Initial load
updateComparison();
