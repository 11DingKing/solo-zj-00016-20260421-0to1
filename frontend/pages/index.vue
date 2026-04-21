<template>
  <div class="min-h-screen bg-gradient-to-br from-indigo-50 via-white to-purple-50">
    <div class="container mx-auto px-4 py-8 max-w-4xl">
      <header class="text-center mb-12">
        <h1 class="text-4xl font-bold text-gray-800 mb-2">短链接生成器</h1>
        <p class="text-gray-600">输入长链接，生成简短易记的短链接</p>
      </header>

      <section class="bg-white rounded-2xl shadow-lg p-6 mb-8">
        <h2 class="text-xl font-semibold text-gray-700 mb-4">创建短链接</h2>
        <form @submit.prevent="createShortLink" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-600 mb-1">长链接</label>
            <input
              v-model="longUrl"
              type="url"
              placeholder="https://example.com/very/long/url"
              class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition"
              required
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-600 mb-1">过期时间（可选）</label>
            <select
              v-model="expiresInHours"
              class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition"
            >
              <option value="">永不过期</option>
              <option :value="1">1 小时后</option>
              <option :value="24">1 天后</option>
              <option :value="168">7 天后</option>
              <option :value="720">30 天后</option>
            </select>
          </div>
          <button
            type="submit"
            :disabled="isLoading"
            class="w-full bg-indigo-600 hover:bg-indigo-700 text-white font-semibold py-3 px-6 rounded-lg transition disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <span v-if="isLoading">生成中...</span>
            <span v-else>生成短链接</span>
          </button>
        </form>

        <div v-if="error" class="mt-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-600">
          {{ error }}
        </div>

        <div
          v-if="createdLink"
          class="mt-6 p-6 bg-gradient-to-r from-indigo-50 to-purple-50 rounded-xl border border-indigo-100"
        >
          <h3 class="text-lg font-semibold text-gray-700 mb-4">短链接已生成！</h3>
          <div class="flex flex-col md:flex-row items-start md:items-center gap-4">
            <div class="flex-1">
              <p class="text-sm text-gray-500 mb-1">短链接</p>
              <div class="flex items-center gap-2">
                <a
                  :href="createdLink.short_url"
                  target="_blank"
                  class="text-indigo-600 hover:text-indigo-800 font-medium text-lg break-all"
                >
                  {{ createdLink.short_url }}
                </a>
                <button
                  @click="copyToClipboard(createdLink.short_url)"
                  class="p-2 text-gray-500 hover:text-indigo-600 hover:bg-indigo-50 rounded-lg transition"
                  title="复制链接"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
              </div>
              <p class="text-sm text-gray-500 mt-2">
                原始链接：{{ createdLink.original_url }}
              </p>
            </div>
            <div class="flex-shrink-0">
              <canvas ref="qrCanvas" class="bg-white p-2 rounded-lg shadow-sm"></canvas>
            </div>
          </div>
        </div>
      </section>

      <section class="bg-white rounded-2xl shadow-lg p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-semibold text-gray-700">我的短链接</h2>
          <button
            @click="loadUserLinks"
            class="text-sm text-indigo-600 hover:text-indigo-800"
          >
            刷新
          </button>
        </div>

        <div v-if="userLinks.length === 0" class="text-center py-12 text-gray-500">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 mx-auto mb-4 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
          </svg>
          <p>还没有创建任何短链接</p>
        </div>

        <div v-else class="space-y-4">
          <div
            v-for="link in userLinks"
            :key="link.short_code"
            class="border border-gray-200 rounded-xl p-4 hover:border-indigo-300 hover:shadow-md transition"
          >
            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <a
                    :href="link.short_url"
                    target="_blank"
                    class="text-indigo-600 hover:text-indigo-800 font-medium truncate"
                  >
                    {{ link.short_url }}
                  </a>
                  <button
                    @click="copyToClipboard(link.short_url)"
                    class="p-1 text-gray-400 hover:text-indigo-600 rounded"
                    title="复制"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                  </button>
                </div>
                <p class="text-sm text-gray-500 truncate">{{ link.original_url }}</p>
                <div class="flex items-center gap-4 mt-2 text-xs text-gray-400">
                  <span>点击: {{ link.total_clicks }}</span>
                  <span>创建于: {{ formatDate(link.created_at) }}</span>
                  <span v-if="link.expires_at">过期: {{ formatDate(link.expires_at) }}</span>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <button
                  @click="showStats(link)"
                  class="px-4 py-2 text-sm bg-indigo-50 text-indigo-600 hover:bg-indigo-100 rounded-lg transition"
                >
                  统计
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <div
        v-if="statsModalVisible"
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
        @click.self="closeStatsModal"
      >
        <div class="bg-white rounded-2xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto">
          <div class="p-6 border-b border-gray-100 flex items-center justify-between">
            <h3 class="text-xl font-semibold text-gray-800">链接统计</h3>
            <button
              @click="closeStatsModal"
              class="p-2 hover:bg-gray-100 rounded-lg transition"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="p-6">
            <div v-if="statsLoading" class="text-center py-8 text-gray-500">
              加载中...
            </div>

            <div v-else-if="statsError" class="text-center py-8 text-red-500">
              {{ statsError }}
            </div>

            <div v-else>
              <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                <div class="bg-gradient-to-br from-indigo-50 to-indigo-100 p-4 rounded-xl">
                  <p class="text-sm text-indigo-600">总点击数</p>
                  <p class="text-3xl font-bold text-indigo-700">{{ stats.total_clicks }}</p>
                </div>
              </div>

              <div class="mb-6">
                <h4 class="text-lg font-medium text-gray-700 mb-3">最近 7 天点击趋势</h4>
                <div class="bg-gray-50 rounded-xl p-4" style="height: 300px;">
                  <Line v-if="stats.daily_clicks.length > 0" :data="chartData" :options="chartOptions" />
                  <div v-else class="flex items-center justify-center h-full text-gray-400">
                    暂无数据
                  </div>
                </div>
              </div>

              <div>
                <h4 class="text-lg font-medium text-gray-700 mb-3">来源 Top 5</h4>
                <div class="bg-gray-50 rounded-xl p-4">
                  <div v-if="stats.top_referers.length === 0" class="text-center py-4 text-gray-400">
                    暂无数据
                  </div>
                  <div v-else class="space-y-2">
                    <div
                      v-for="(referer, index) in stats.top_referers"
                      :key="referer.referer"
                      class="flex items-center gap-3 p-3 bg-white rounded-lg"
                    >
                      <span class="flex-shrink-0 w-8 h-8 flex items-center justify-center bg-indigo-100 text-indigo-600 rounded-full text-sm font-medium">
                        {{ index + 1 }}
                      </span>
                      <div class="flex-1 min-w-0">
                        <p class="text-sm font-medium text-gray-700 truncate">{{ referer.referer || '直接访问' }}</p>
                      </div>
                      <span class="flex-shrink-0 text-sm text-gray-500">{{ referer.count }} 次</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed, nextTick } from 'vue';
import QRCode from 'qrcode';
import { Line } from 'vue-chartjs';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
} from 'chart.js';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
);

const runtimeConfig = useRuntimeConfig();
const apiBase = runtimeConfig.public.apiBase;

const longUrl = ref('');
const expiresInHours = ref('');
const isLoading = ref(false);
const error = ref('');
const createdLink = ref(null);
const qrCanvas = ref(null);
const userLinks = ref([]);
const statsModalVisible = ref(false);
const selectedLink = ref(null);
const stats = ref(null);
const statsLoading = ref(false);
const statsError = ref('');

const chartData = computed(() => {
  if (!stats.value) return { labels: [], datasets: [] };
  
  const sortedClicks = [...stats.value.daily_clicks].sort((a, b) => 
    new Date(a.date) - new Date(b.date)
  );
  
  return {
    labels: sortedClicks.map(c => {
      const date = new Date(c.date);
      return `${date.getMonth() + 1}/${date.getDate()}`;
    }),
    datasets: [
      {
        label: '点击数',
        backgroundColor: 'rgba(99, 102, 241, 0.1)',
        borderColor: 'rgb(99, 102, 241)',
        borderWidth: 2,
        data: sortedClicks.map(c => c.count),
        tension: 0.4,
        fill: true,
        pointBackgroundColor: 'rgb(99, 102, 241)',
        pointBorderColor: '#fff',
        pointBorderWidth: 2,
        pointRadius: 4,
      },
    ],
  };
});

const chartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      display: false,
    },
    tooltip: {
      backgroundColor: 'rgba(0, 0, 0, 0.8)',
      padding: 12,
      titleFont: {
        size: 14,
      },
      bodyFont: {
        size: 13,
      },
    },
  },
  scales: {
    y: {
      beginAtZero: true,
      ticks: {
        precision: 0,
      },
      grid: {
        color: 'rgba(0, 0, 0, 0.05)',
      },
    },
    x: {
      grid: {
        display: false,
      },
    },
  },
}));

const generateQRCode = async () => {
  if (!qrCanvas.value || !createdLink.value) return;
  await nextTick();
  const canvas = qrCanvas.value;
  if (canvas) {
    try {
      await QRCode.toCanvas(canvas, createdLink.value.short_url, {
        width: 128,
        margin: 2,
        color: {
          dark: '#1e1b4b',
          light: '#ffffff',
        },
      });
    } catch (e) {
      console.error('QR code error:', e);
    }
  }
};

const createShortLink = async () => {
  isLoading.value = true;
  error.value = '';
  createdLink.value = null;

  try {
    const response = await fetch(`${apiBase}/api/links`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        original_url: longUrl.value,
        expires_in_hours: expiresInHours.value ? parseInt(expiresInHours.value) : null,
      }),
      credentials: 'include',
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.error || `HTTP error! status: ${response.status}`);
    }

    createdLink.value = await response.json();
    longUrl.value = '';
    expiresInHours.value = '';
    
    await nextTick();
    await generateQRCode();
    
    await loadUserLinks();
  } catch (e) {
    error.value = e.message || '创建短链接失败，请重试';
  } finally {
    isLoading.value = false;
  }
};

const loadUserLinks = async () => {
  try {
    const response = await fetch(`${apiBase}/api/links`, {
      credentials: 'include',
    });
    
    if (response.ok) {
      const data = await response.json();
      userLinks.value = data.links || [];
    }
  } catch (e) {
    console.error('Failed to load user links:', e);
  }
};

const showStats = async (link) => {
  selectedLink.value = link;
  statsModalVisible.value = true;
  statsLoading.value = true;
  statsError.value = '';
  stats.value = null;

  try {
    const response = await fetch(`${apiBase}/api/links/${link.short_code}/stats`);
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    stats.value = await response.json();
  } catch (e) {
    statsError.value = e.message || '加载统计数据失败';
  } finally {
    statsLoading.value = false;
  }
};

const closeStatsModal = () => {
  statsModalVisible.value = false;
  selectedLink.value = null;
  stats.value = null;
};

const copyToClipboard = async (text) => {
  try {
    await navigator.clipboard.writeText(text);
  } catch (e) {
    console.error('Copy failed:', e);
  }
};

const formatDate = (dateStr) => {
  if (!dateStr) return '';
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
};

onMounted(() => {
  loadUserLinks();
});
</script>
