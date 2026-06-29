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
  footerThemeDark: string;
  footerThemeLight: string;
  footerThemeSystem: string;
  footerLang: string;
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
  const [theme, setTheme] = createSignal<'dark' | 'light' | 'system'>(
    (localStorage.getItem('theme') as 'dark' | 'light' | 'system') || 'system'
  );
  const [t, setT] = createSignal<TranslationDict | null>(null);
  const [comparison, setComparison] = createSignal<ComparisonData | null>(null);
  const [metrics, setMetrics] = createSignal<MetricsData | null>(null);
  const [sysInfo, setSysInfo] = createSignal<SysInfoData | null>(null);
  const [isMobile, setIsMobile] = createSignal<boolean>(window.innerWidth < 768);

  // Sync theme to document element
  createEffect(() => {
    const activeTheme = theme();
    localStorage.setItem('theme', activeTheme);
    
    const resolveActiveTheme = (val: 'dark' | 'light' | 'system'): 'dark' | 'light' => {
      if (val === 'system') {
        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
      }
      return val;
    };
    
    document.documentElement.setAttribute('data-theme', resolveActiveTheme(activeTheme));
  });

  // Fetch translation dictionary
  createEffect(() => {
    const activeLang = lang();
    localStorage.setItem('lang', activeLang);
    
    const loadTranslations = async () => {
      try {
        const res = await fetch(`/locales/${activeLang}.json`);
        if (res.ok) {
          const dict = (await res.json()) as TranslationDict;
          setT(dict);
        }
      } catch (err) {
        console.error('Failed to load translations:', err);
      }
    };
    
    loadTranslations();
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

    const handleResize = () => {
      setIsMobile(window.innerWidth < 768);
    };
    window.addEventListener('resize', handleResize);

    // Listen to system theme preference changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleSystemThemeChange = () => {
      if (theme() === 'system') {
        document.documentElement.setAttribute('data-theme', mediaQuery.matches ? 'dark' : 'light');
      }
    };
    mediaQuery.addEventListener('change', handleSystemThemeChange);

    return () => {
      clearInterval(interval);
      window.removeEventListener('resize', handleResize);
      mediaQuery.removeEventListener('change', handleSystemThemeChange);
    };
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
    const current = theme();
    if (current === 'system') {
      setTheme('light');
    } else if (current === 'light') {
      setTheme('dark');
    } else {
      setTheme('system');
    }
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
      <div class="container animate-fade-in" style="padding-bottom: 6rem; padding-top: 3rem;">

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
                  return (
                    <div class="delivery-option" id={`opt-${p.key}`}>
                      <div class="option-container">
                        <div class="option-header">
                          <div class="provider-info">
                            <img class="provider-logo" src={`/api/icon?provider=${p.key}`} alt={meta.name} />
                            <div class="provider-meta">
                              <span class="provider-name">{meta.name}</span>
                              <div class={`promo-badge ${meta.badgeClass}`}>
                                {t()![meta.promoKey]}
                              </div>
                            </div>
                          </div>
                        </div>
                        <div class="option-details">
                          <div class="formula-box">
                            <div class="formula-list">
                              <div class="formula-row">
                                <span class="formula-label">{t()!.foodOrder}</span>
                                <span class="formula-val">250</span>
                              </div>
                              <div class="formula-row">
                                <span class="formula-label">{t()!.deliveryFee}</span>
                                <span class="formula-val">+{deliveryFee.toFixed(0)}</span>
                              </div>
                              <div class="formula-row">
                                <span class="formula-label">{t()!.discount}</span>
                                <span class="formula-val discount-val">-{discount.toFixed(0)}</span>
                              </div>
                              <hr class="formula-divider" />
                              <div class="formula-row total-row">
                                <span class="formula-label">{t()!.totalCost}</span>
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

        <footer style="margin-top: 5rem; border-top: 1px solid var(--border-color); padding: 2rem 0 3rem 0; font-size: 0.75rem; color: var(--text-secondary);">
          <Show when={sysInfo()}>
            {(info) => {
              const data = info();
              return (
                <div style="font-size: 0.7rem; line-height: 1.5; margin-bottom: 2rem; opacity: 0.65;">
                  {t()!.sysHost}: {data.hostname} ({data.hypervisor}) | {t()!.sysOs}: {data.os}
                  <br />
                  {t()!.sysKernel}: {data.kernel} | {t()!.sysUptime}: {data.uptime}
                  <br />
                  {t()!.sysCpu}: {data.cpu_model} ({data.cpu_cores} {t()!.sysCores} @ {data.cpu_speed})
                  <br />
                  {t()!.sysFlags}: <code style="color: var(--accent-primary);">{data.cpu_flags}</code>
                  <br />
                  {t()!.sysCache}: L1-D {data.l1_data} | L1-I {data.l1_inst} | L2 {data.l2} | L3 {data.l3} | {t()!.sysLine}: {data.cache_line}B
                  <br />
                  {t()!.sysPage}: {data.page_size}B | {t()!.sysNuma}: {data.numa_nodes} | {t()!.sysRam}: {data.mem_total}
                  <br />
                  {t()!.sysLoad}: {data.loadavg}
                </div>
              );
            }}
          </Show>

          {/* Enterprise Docked Footer Bar */}
          <div class="footer-bar">
            <div class="footer-bar-left">
              {/* Globe Icon + Language Switcher */}
              <button class="footer-btn" onClick={toggleLanguage}>
                <svg viewBox="0 0 24 24" class="footer-icon">
                  <circle cx="12" cy="12" r="10"></circle>
                  <line x1="2" y1="12" x2="22" y2="12"></line>
                  <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                </svg>
                {t()!.footerLang}
              </button>

              {/* Theme switcher */}
              <button class="footer-btn" onClick={toggleTheme}>
                {theme() === 'system' ? (
                  <>
                    <svg viewBox="0 0 24 24" class="footer-icon">
                      <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
                      <line x1="8" y1="21" x2="16" y2="21"></line>
                      <line x1="12" y1="17" x2="12" y2="21"></line>
                    </svg>
                    {t()!.footerThemeSystem}
                  </>
                ) : theme() === 'light' ? (
                  <>
                    <svg viewBox="0 0 24 24" class="footer-icon">
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
                    {t()!.footerThemeLight}
                  </>
                ) : (
                  <>
                    <svg viewBox="0 0 24 24" class="footer-icon">
                      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                    </svg>
                    {t()!.footerThemeDark}
                  </>
                )}
              </button>
              
              {/* Diagnostics telemetry */}
              <span class="footer-diagnostics">
                {metrics() ? `RAM: ${metrics()!.mem_current_mb.toFixed(2)} MB | CPU: ${metrics()!.cpu_percent.toFixed(2)}%` : 'RAM: -- | CPU: --'}
              </span>
            </div>

            <div class="footer-bar-right">
              <span class="footer-link">
                {lang() === 'en' ? 'Privacy' : 'ความเป็นส่วนตัว'}
              </span>
              <span class="footer-link">
                {lang() === 'en' ? 'Terms' : 'ข้อตกลงการใช้งาน'}
              </span>
              <span class="footer-copyright">
                © Khum 2026
              </span>
            </div>
          </div>
        </footer>
      </div>
    </Show>
  );
}

export default App;
