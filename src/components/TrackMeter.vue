<template>
  <div class="track-meter">
    <!-- Stereo: 2 VU meters affiancati -->
    <div v-if="isStereo" class="flex gap-0.5">
      <VuMeter 
        :level="levelL" 
        :label="'L'" 
        :height="height" 
        :width="8" 
        :segments="30"
        :showValue="false" 
      />
      <VuMeter 
        :level="levelR" 
        :label="'R'" 
        :height="height" 
        :width="8" 
        :segments="30"
        :showValue="false" 
      />
    </div>
    
    <!-- Mono: 1 VU meter più largo (stesso ingombro totale) -->
    <VuMeter 
      v-else
      :level="levelL" 
      :label="''" 
      :height="height" 
      :width="18" 
      :segments="30"
      :showValue="false" 
    />
  </div>
</template>

<script setup lang="ts">
import VuMeter from './VuMeter.vue'

interface Props {
  levelL: number    // Left or mono level in dB
  levelR?: number   // Right level in dB (only for stereo)
  isStereo: boolean // True for stereo, false for mono
  height?: number   // Height of the meter(s)
}

const props = withDefaults(defineProps<Props>(), {
  levelR: -60,
  height: 220
})
</script>

<style scoped>
.track-meter {
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
