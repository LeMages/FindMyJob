<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const message = ref("");

onMounted(async () => {
  message.value = await invoke<string>("greet");
});
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-gray-50">
    <p class="text-3xl font-bold text-gray-800">{{ message }}</p>
  </div>
</template>
