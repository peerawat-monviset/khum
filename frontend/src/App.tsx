import { createSignal, createEffect, onMount, For, Show } from 'solid-js';

interface ProviderData {
  key: 'grab' | 'lineman' | 'shopee' | 'robinhood';
  name: string;
  final: number;
  original: number;
}

interface ComparisonData {
  best: 'grab' | 'lineman' | 'shopee' | 'robinhood';
  providers: ProviderData[];
}

interface MetricsData {
  mem_current_mb: number;
  mem_limit_mb: number;
  cpu_percent: number;
}

interface SysInfoData {
  os: string;
  kernel: string;
  hostname: string;
  hypervisor: string;
  cpu_model: string;
  cpu_speed: string;
  cpu_cores: number;
  cpu_flags: string;
  l1_data: string;
  l1_inst: string;
  l2: string;
  l3: string;
  cache_line: number;
  numa_nodes: number;
  page_size: number;
  mem_total: string;
  uptime: string;
  loadavg: string;
}

interface TranslationDict {
  title: string;
  unit: string;
  foodOrder: string;
  deliveryFee: string;
  discount: string;
  totalCost: string;
  server: string;
  loadingVm: string;
  grabPromo: string;
  linemanPromo: string;
  shopeePromo: string;
  robinhoodPromo: string;
  sysHost: string;
  sysOs: string;
  sysKernel: string;
  sysUptime: string;
  sysCpu: string;
  sysCores: string;
  sysSpeed: string;
  sysFlags: string;
  sysCache: string;
  sysLine: string;
  sysPage: string;
  sysNuma: string;
  sysRam: string;
  sysLoad: string;
}

type ProviderMetaMap = {
  [key in 'grab' | 'lineman' | 'shopee' | 'robinhood']: {
    name: string;
    badgeClass: string;
    promoKey: keyof TranslationDict;
  };
};

function App() {
  const [lang, setLang] = createSignal<'en' | 'th'>((localStorage.getItem('lang') as 'en' | 'th') || 'en');
  const [theme, setTheme] = createSignal<'dark' | 'light'>(
    (localStorage.getItem('theme') as 'dark' | 'light') || 
    (window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark')
  );
  const [t, setT] = createSignal<TranslationDict | null>(null);
  const [comparison, setComparison] = createSignal<ComparisonData | null>(null);
  const [metrics, setMetrics] = createSignal<MetricsData | null>(null);
  const [sysInfo, setSysInfo] = createSignal<SysInfoData | null>(null);

  // Sync theme to document element
  createEffect(() => {
    const activeTheme = theme();
    document.documentElement.setAttribute('data-theme', activeTheme);
    localStorage.setItem('theme', activeTheme);
  });

  // Fetch translation dictionary
  createEffect(async () => {
    const activeLang = lang();
    localStorage.setItem('lang', activeLang);
    try {
      const res = await fetch(`/locales/${activeLang}.json`);
      if (res.ok) {
        const dict = (await res.json()) as TranslationDict;
        setT(dict);
      }
    } catch (err) {
      console.error('Failed to load translations:', err);
    }
  });

  // Fetch calculation results
  const fetchComparison = async () => {
    try {
      const res = await fetch('/api/calculate?basket=250&distance=3.5');
      if (res.ok) {
        const data = (await res.json()) as ComparisonData;
        setComparison(data);
      }
    } catch (err) {
      console.error('Failed to fetch calculation:', err);
    }
  };

  // Fetch VM specifications
  const fetchSysInfo = async () => {
    try {
      const res = await fetch('/api/sysinfo');
      if (res.ok) {
        const data = (await res.json()) as SysInfoData;
        setSysInfo(data);
      }
    } catch (err) {
      console.error('Failed to fetch sysinfo:', err);
    }
  };

  // Fetch memory and CPU metrics
  const fetchMetrics = async () => {
    try {
      const res = await fetch('/api/metrics');
      if (res.ok) {
        const data = (await res.json()) as MetricsData;
        setMetrics(data);
      }
    } catch (err) {
      console.error('Failed to fetch metrics:', err);
    }
  };

  onMount(() => {
    fetchComparison();
    fetchSysInfo();
    fetchMetrics();
    const interval = setInterval(fetchMetrics, 5000);
    return () => clearInterval(interval);
  });

  // Trigger calculation and info refresh when translation loads
  createEffect(() => {
    if (t()) {
      fetchComparison();
      fetchMetrics();
    }
  });

  const toggleLanguage = () => {
    setLang(lang() === 'en' ? 'th' : 'en');
  };

  const toggleTheme = () => {
    setTheme(theme() === 'dark' ? 'light' : 'dark');
  };

  // Providers order matched to market share: Grab, LINE MAN, ShopeeFood, Robinhood
  const providerMeta: ProviderMetaMap = {
    grab: { name: 'GrabFood', badgeClass: 'badge-grab', promoKey: 'grabPromo' },
    lineman: { name: 'LINE MAN', badgeClass: 'badge-lineman', promoKey: 'linemanPromo' },
    shopee: { name: 'ShopeeFood', badgeClass: 'badge-shopee', promoKey: 'shopeePromo' },
    robinhood: { name: 'Robinhood', badgeClass: 'badge-robinhood', promoKey: 'robinhoodPromo' }
  };

  return (
    <Show when={t()}>
      {(translation) => {
        const activeT = translation();
        return (
          <div class="container animate-fade-in">
            <header>
              <div class="header-controls">
                <button class="control-btn" onClick={toggleLanguage}>
                  {lang() === 'en' ? 'ไทย' : 'EN'}
                </button>
                <button class="control-btn" onClick={toggleTheme} aria-label="Toggle Theme">
                  <span style="display: flex; align-items: center; justify-content: center;">
                    {theme() === 'dark' ? (
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="width: 20px; height: 20px;">
                        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                      </svg>
                    ) : (
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="width: 20px; height: 20px;">
                        <circle cx="12" cy="12" r="5"></circle>
                        <line x1="12" y1="1" x2="12" y2="3"></line>
                        <line x1="12" y1="21" x2="12" y2="23"></line>
                        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                        <line x1="1" y1="12" x2="3" y2="12"></line>
                        <line x1="21" y1="12" x2="23" y2="12"></line>
                        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                      </svg>
                    )}
                  </span>
                </button>
              </div>
              <h1>{activeT.title}</h1>
              <div class="unit-indicator">{activeT.unit}</div>
            </header>

            <div class="comparison-list">
              <Show 
                when={comparison()} 
                fallback={
                  <div style="text-align: center; color: var(--text-secondary); margin: 2rem 0;">
                    Loading Options...
                  </div>
                }
              >
                {(compData) => (
                  <For each={compData().providers}>
                    {(p) => {
                      const meta = providerMeta[p.key];
                      const deliveryFee = p.original - 250;
                      const discount = p.original - p.final;
                      const isBest = compData().best === p.key;

                      return (
                        <div class={`delivery-option ${isBest ? 'best-deal' : ''}`} id={`opt-${p.key}`}>
                          <div class="option-container">
                            <div class="option-header">
                              <div class="provider-info">
                                <img class="provider-logo" src={`/api/icon?provider=${p.key}`} alt={meta.name} />
                                <div class="provider-meta">
                                  <span class="provider-name">{meta.name}</span>
                                  <div class={`promo-badge ${meta.badgeClass}`}>
                                    {activeT[meta.promoKey]}
                                  </div>
                                </div>
                              </div>
                              <div class="price-details">
                                <div class="final-price">{p.final.toFixed(0)}</div>
                              </div>
                            </div>
                            <div class="option-details">
                              <div class="formula-box">
                                <div class="formula-list">
                                  <div class="formula-row">
                                    <span class="formula-label">{activeT.foodOrder}</span>
                                    <span class="formula-val">250</span>
                                  </div>
                                  <div class="formula-row">
                                    <span class="formula-label">{activeT.deliveryFee}</span>
                                    <span class="formula-val">+{deliveryFee.toFixed(0)}</span>
                                  </div>
                                  <div class="formula-row">
                                    <span class="formula-label">{activeT.discount}</span>
                                    <span class="formula-val discount-val">-{discount.toFixed(0)}</span>
                                  </div>
                                  <hr class="formula-divider" />
                                  <div class="formula-row total-row">
                                    <span class="formula-label">{activeT.totalCost}</span>
                                    <span class="formula-val">{p.final.toFixed(0)}</span>
                                  </div>
                                </div>
                              </div>
                            </div>
                          </div>
                        </div>
                      );
                    }}
                  </For>
                )}
              </Show>
            </div>

            <footer style="margin-top: 3rem; text-align: center; font-size: 0.8rem; color: var(--text-secondary); opacity: 0.7;">
              <p>
                <span>{activeT.server}</span> RAM{' '}
                <span>
                  {metrics() ? `${metrics()!.mem_current_mb.toFixed(2)} MB / ${metrics()!.mem_limit_mb.toFixed(2)} MB` : '0.00 MB / 0 MB'}
                </span>{' '}
                | CPU <span>{metrics() ? `${metrics()!.cpu_percent.toFixed(2)}%` : '0.00%'}</span>
              </p>
              <Show when={sysInfo()} fallback={<p style="font-size: 0.7rem; margin-top: 0.5rem; line-height: 1.4;">{activeT.loadingVm}</p>}>
                {(info) => {
                  const data = info();
                  return (
                    <p style="font-size: 0.7rem; margin-top: 0.5rem; line-height: 1.4;">
                      {activeT.sysHost}: {data.hostname} ({data.hypervisor}) | {activeT.sysOs}: {data.os}
                      <br />
                      {activeT.sysKernel}: {data.kernel} | {activeT.sysUptime}: {data.uptime}
                      <br />
                      {activeT.sysCpu}: {data.cpu_model} ({data.cpu_cores} {activeT.sysCores} @ {data.cpu_speed})
                      <br />
                      {activeT.sysFlags}: <code style="color: var(--accent-primary);">{data.cpu_flags}</code>
                      <br />
                      {activeT.sysCache}: L1-D {data.l1_data} | L1-I {data.l1_inst} | L2 {data.l2} | L3 {data.l3} | {activeT.sysLine}: {data.cache_line}B
                      <br />
                      {activeT.sysPage}: {data.page_size}B | {activeT.sysNuma}: {data.numa_nodes} | {activeT.sysRam}: {data.mem_total}
                      <br />
                      {activeT.sysLoad}: {data.loadavg}
                    </p>
                  );
                }}
              </Show>
            </footer>
          </div>
        );
      }}
    </Show>
  );
}

export default App;
