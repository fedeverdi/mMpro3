<template>
  <div class="flex flex-col items-center gap-0.5">
    <button
      @click="toggle"
      :class="[
        'px-1.5 py-[0.1rem] rounded font-bold text-[0.5rem] transition-all duration-200 border',
        modelValue
          ? 'bg-yellow-600 border-yellow-500 text-white shadow-md shadow-yellow-600/30'
          : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600 hover:border-gray-500'
      ]"
      :title="modelValue ? 'PAD: -20dB attivo' : 'PAD: -20dB disattivo'"
    >
      PAD
    </button>
    <div class="text-[7px] text-gray-400 font-medium">26dB</div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  modelValue: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

function toggle() {
  emit('update:modelValue', !props.modelValue)
}
</script>
