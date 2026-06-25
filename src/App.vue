<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface DbStatus {
  db_path: string;
  tables: string[];
}

const dbStatus = ref<DbStatus | null>(null);
const dbError = ref<string>("");

onMounted(async () => {
  try {
    dbStatus.value = await invoke<DbStatus>("db_status");
  } catch (e) {
    dbError.value = String(e);
  }
});
</script>

<template>
  <div class="min-h-screen bg-gray-50 p-8">
    <h1 class="mb-6 text-2xl font-bold text-gray-800">FindMyJob</h1>

    <div v-if="dbStatus" class="rounded-lg bg-white p-6 shadow">
      <h2 class="mb-1 text-sm font-semibold uppercase tracking-wide text-gray-500">
        Base de données
      </h2>
      <p class="mb-4 font-mono text-sm text-gray-700">{{ dbStatus.db_path }}</p>

      <h2 class="mb-2 text-sm font-semibold uppercase tracking-wide text-gray-500">
        Tables ({{ dbStatus.tables.length }})
      </h2>
      <ul class="space-y-1">
        <li
          v-for="table in dbStatus.tables"
          :key="table"
          class="font-mono text-sm text-green-700"
        >
          ✓ {{ table }}
        </li>
      </ul>
    </div>

    <div v-else-if="dbError" class="rounded-lg bg-red-50 p-4 font-mono text-sm text-red-700">
      {{ dbError }}
    </div>

    <div v-else class="text-sm text-gray-400">Chargement…</div>
  </div>
</template>
