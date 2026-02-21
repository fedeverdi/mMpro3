<template>
  <div class="flex flex-col items-center gap-0.5">
    <button
      @click="toggle"
      :class="[
        'px-1 py-0.5 rounded font-bold text-[8px] transition-all duration-200 border',
        modelValue
          ? 'bg-blue-600 border-blue-500 text-white shadow-md shadow-blue-600/30'
          : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600 hover:border-gray-500'
      ]"
      :title="modelValue ? 'HPF: 80Hz attivo' : 'HPF: 80Hz disattivo'"
    >
      HPF
    </button>
    <div class="text-[8px] text-gray-400 font-medium">80Hz</div>
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
