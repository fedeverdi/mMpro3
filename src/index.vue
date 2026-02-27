<template>
    <div class="mixer-app min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-black flex flex-col">
        <!-- Header -->
        <header class="bg-black/50 backdrop-blur-sm border-b border-gray-700 px-4 py-2">
            <div class="flex items-center justify-between gap-4 flex-wrap">
                <div class="flex items-center gap-2">
                    <img src="./assets/logo_no_scritta.svg" alt="mMpro3" class="h-8" />

                    <!-- Quick Scene Access -->
                    <template v-if="pinnedScenes.length > 0">
                        <div class="w-px h-6 bg-gray-600"></div>
                        <div class="flex gap-1 items-center">
                            <span class="text-[10px] text-gray-400 font-semibold uppercase">Quick scenes:</span>
                            <button v-for="scene in pinnedScenes" :key="scene.id" @click="handleLoadScene(scene.id)"
                                class="px-2 py-0.5 text-[0.65rem] rounded transition-all uppercase border" :class="scene.id === currentSceneId
                                    ? 'border-green-500 bg-green-500/20 text-green-400'
                                    : 'border-gray-600 hover:border-yellow-500 hover:bg-yellow-500/10 text-gray-300 hover:text-yellow-400'"
                                :title="`Load scene: ${scene.name}`">
                                {{ scene.name }}
                            </button>
                        </div>
                    </template>
                </div>
                <div class="flex gap-2 items-center flex-wrap">
                    <button @click="showAudioFlowModal = true"
                        class="px-3 py-1.5 border border-gray-600 hover:border-purple-500 hover:bg-purple-500/10 rounded text-xs font-semibold text-gray-300 hover:text-purple-400 transition-all flex items-center gap-1.5">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                        </svg>
                        Signal Flow
                    </button>

                    <button @click="showScenesModal = true"
                        class="px-3 py-1.5 border border-gray-600 hover:border-green-500 hover:bg-green-500/10 rounded text-xs font-semibold text-gray-300 hover:text-green-400 transition-all flex items-center gap-1.5">
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" class="h-3.5 w-3.5" fill="currentColor">
                            <path
                                d="M149.333 216v80c0 13.255-10.745 24-24 24H24c-13.255 0-24-10.745-24-24v-80c0-13.255 10.745-24 24-24h101.333c13.255 0 24 10.745 24 24zM0 376v80c0 13.255 10.745 24 24 24h101.333c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H24c-13.255 0-24 10.745-24 24zM125.333 32H24C10.745 32 0 42.745 0 56v80c0 13.255 10.745 24 24 24h101.333c13.255 0 24-10.745 24-24V56c0-13.255-10.745-24-24-24zm80 448H488c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24v80c0 13.255 10.745 24 24 24zm-24-424v80c0 13.255 10.745 24 24 24H488c13.255 0 24-10.745 24-24V56c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24zm24 264H488c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24v80c0 13.255 10.745 24 24 24z" />
                        </svg>
                        Scenes
                    </button>

                    <!-- File Manager Button -->
                    <button @click="showFileManager = true"
                        class="px-3 py-1.5 border border-gray-600 hover:border-blue-500 hover:bg-blue-500/10 rounded text-xs font-semibold text-gray-300 hover:text-blue-400 transition-all flex items-center gap-1.5">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                        </svg>
                        Library
                    </button>

                    <button @click="handleClearScene"
                        class="px-3 py-1.5 border border-gray-600 hover:border-orange-500 hover:bg-orange-500/10 rounded text-xs font-semibold text-gray-300 hover:text-orange-400 transition-all flex items-center gap-1.5"
                        title="Clear mixer - Reload page">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="currentColor" viewBox="0 0 640 512">
                            <path
                                d="M256.47 216.77l86.73 109.18s-16.6 102.36-76.57 150.12C206.66 523.85 0 510.19 0 510.19s3.8-23.14 11-55.43l94.62-112.17c3.97-4.7-.87-11.62-6.65-9.5l-60.4 22.09c14.44-41.66 32.72-80.04 54.6-97.47 59.97-47.76 163.3-40.94 163.3-40.94zM636.53 31.03l-19.86-25c-5.49-6.9-15.52-8.05-22.41-2.56l-232.48 177.8-34.14-42.97c-5.09-6.41-15.14-5.21-18.59 2.21l-25.33 54.55 86.73 109.18 58.8-12.45c8-1.69 11.42-11.2 6.34-17.6l-34.09-42.92 232.48-177.8c6.89-5.48 8.04-15.53 2.55-22.44z" />
                        </svg>
                        Clear
                    </button>

                    <div class="w-px h-6 bg-gray-600"></div>

                    <div class="relative -mt-[3px]">
                        <button @click="handleAddButtonClick"
                            class="mt-1 px-3 h-full py-1.5 border border-gray-600 hover:border-emerald-500 hover:bg-emerald-500/10 rounded text-xs font-semibold text-gray-300 hover:text-emerald-400 transition-all flex items-center gap-1.5">
                            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                            </svg>
                            Add
                        </button>

                        <!-- Dropdown Menu -->
                        <div v-if="showAddTrackMenu"
                            class="absolute top-full left-0 mt-1 w-32 bg-gray-800 border border-gray-600 rounded shadow-lg z-[1000]">
                            <button @click="addTrackOfType('audio')"
                                class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2">
                                <div class="flex">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white"
                                        viewBox="0 0 256 512">
                                        <path
                                            d="M96 496V16c0-8.8-7.2-16-16-16H48c-8.8 0-16 7.2-16 16v480c0 8.8 7.2 16 16 16h32c8.8 0 16-7.2 16-16zm128 0V16c0-8.8-7.2-16-16-16h-32c-8.8 0-16 7.2-16 16v480c0 8.8 7.2 16 16 16h32c8.8 0 16-7.2 16-16z" />
                                    </svg>
                                </div>
                                Audio Track
                            </button>
                            <button @click="addTrackOfType('signal')"
                                class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2">
                                <div class="flex">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white"
                                        viewBox="0 0 640 512">
                                        <path
                                            d="M476 480H324a36 36 0 0 1-36-36V96h-96v156a36 36 0 0 1-36 36H16a16 16 0 0 1-16-16v-32a16 16 0 0 1 16-16h112V68a36 36 0 0 1 36-36h152a36 36 0 0 1 36 36v348h96V260a36 36 0 0 1 36-36h140a16 16 0 0 1 16 16v32a16 16 0 0 1-16 16H512v156a36 36 0 0 1-36 36z" />
                                    </svg>
                                </div>
                                Signal Track
                            </button>
                            <template v-if="buildLimits.maxSubgroups > 0">
                                <div class="h-px bg-gray-600 my-1"></div>
                                <button @click="addSubgroup(); showAddTrackMenu = false"
                                    class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2">
                                    <div class="flex">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white"
                                            viewBox="0 0 512 512">
                                            <path
                                                d="M12.41 148.02l232.94 105.67c6.8 3.09 14.49 3.09 21.29 0l232.94-105.67c16.55-7.51 16.55-32.52 0-40.03L266.65 2.31a25.607 25.607 0 0 0-21.29 0L12.41 107.98c-16.55 7.51-16.55 32.53 0 40.04zm487.18 88.28l-58.09-26.33-161.64 73.27c-7.56 3.43-15.59 5.17-23.86 5.17s-16.29-1.74-23.86-5.17L70.51 209.97l-58.1 26.33c-16.55 7.5-16.55 32.5 0 40l232.94 105.59c6.8 3.08 14.49 3.08 21.29 0L499.59 276.3c16.55-7.5 16.55-32.5 0-40zm0 127.8l-57.87-26.23-161.86 73.37c-7.56 3.43-15.59 5.17-23.86 5.17s-16.29-1.74-23.86-5.17L70.29 337.87 12.41 364.1c-16.55 7.5-16.55 32.5 0 40l232.94 105.59c6.8 3.08 14.49 3.08 21.29 0L499.59 404.1c16.55-7.5 16.55-32.5 0-40z" />
                                        </svg>
                                    </div>
                                    Subgroup
                                </button>
                            </template>
                        </div>
                    </div>

                    <button @click="removeTrack(tracks[tracks.length - 1].id)" :disabled="tracks.length <= 1"
                        class="px-3 py-1.5 border border-gray-600 hover:border-red-500 hover:bg-red-500/10 disabled:border-gray-700 disabled:bg-gray-800/50 disabled:cursor-not-allowed rounded text-xs font-semibold text-gray-300 hover:text-red-400 disabled:text-gray-600 transition-all flex items-center gap-1.5">
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
                        </svg>
                        Remove
                    </button>

                    <div class="text-xs text-gray-400">
                        {{ tracks.length }}/{{ buildLimits.maxTracks }}
                    </div>
                </div>
            </div>
        </header>

        <!-- Mixer Console -->
        <main class="flex-1 flex gap-2 p-2 pb-14 overflow-hidden">
            <!-- Audio Tracks Section (flexible) -->
            <div class="tracks-scroll-wrap flex-1 overflow-hidden min-w-0 pb-[2px]">
                <div class="tracks-scroll overflow-x-auto overflow-y-hidden h-full">
                    <div class="flex gap-2 h-full min-w-max">
                        <!-- Loading Skeletons -->
                        <template v-if="!isReady">
                            <div v-for="n in 24" :key="'skeleton-' + n" class="w-[8rem] h-full">
                                <div
                                    class="bg-gray-800 rounded-lg border border-gray-700 p-1 h-full flex flex-col gap-2">
                                    <!-- Header -->
                                    <div class="h-4 bg-gray-700 rounded animate-pulse"></div>
                                    <!-- Controls -->
                                    <div class="h-6 bg-gray-700 rounded animate-pulse"></div>
                                    <div class="h-6 bg-gray-700 rounded animate-pulse"></div>
                                    <div class="flex gap-1">
                                        <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                                        <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                                    </div>
                                    <!-- Knob area -->
                                    <div class="flex-1 flex items-center justify-center">
                                        <div class="w-16 h-16 bg-gray-700 rounded-full animate-pulse"></div>
                                    </div>
                                    <!-- Fader area -->
                                    <div class="flex-1 flex flex-col items-center justify-center gap-2 py-4">
                                        <!-- VU Meter skeleton -->
                                        <div class="w-12 flex-1 bg-gray-700 rounded animate-pulse"></div>
                                        <!-- Fader skeleton -->
                                        <div class="w-8 h-32 bg-gray-700 rounded animate-pulse"></div>
                                    </div>
                                    <!-- Bottom controls -->
                                    <div class="flex gap-1">
                                        <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                                        <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                                    </div>
                                </div>
                            </div>
                        </template>

                        <!-- Audio Tracks -->
                        <template v-else>
                            <div v-for="track in sortedTracks" :key="track.id" 
                                class="w-[8.5rem] h-full mixer-fade-in track-wrapper"
                                :class="{
                                    'dragging': draggedTrackId === track.id,
                                    'drag-over': dragOverTrackId === track.id
                                }"
                                @dragover="handleTrackDragOver(track.id, $event)"
                                @dragleave="handleTrackDragLeave"
                                @drop="handleTrackDrop(track.id)"
                                @dragend="handleTrackDragEnd">
                                <SignalTrack v-if="track.type === 'signal'" 
                                    :ref="el => setTrackRef(track.id, el)"
                                    :trackNumber="track.id"
                                    :order="track.order"
                                    :master-channel="masterChannel" 
                                    :subgroups="subgroups"
                                    :allow-subgroup-routing="buildLimits.allowSubgroupRouting"
                                    :is-dragging="draggedTrackId === track.id"
                                    @soloChange="handleSoloChange" 
                                    @levelUpdate="handleLevelUpdate"
                                    @remove="removeTrack(track.id)"
                                    @drag-start="handleTrackDragStart(track.id)" />
                                <AudioTrack v-else 
                                    :ref="el => setTrackRef(track.id, el)" 
                                    :trackNumber="track.id"
                                    :master-channel="masterChannel" 
                                    :subgroups="subgroups" 
                                    :allow-subgroup-routing="buildLimits.allowSubgroupRouting"
                                    @toggle-arm="toggleTrackArm(track.id)"
                                    @remove="removeTrack(track.id)" />
                            </div>
                        </template>
                    </div>
                </div>
            </div>

            <!-- Right Section (fixed width) -->
            <div class="flex gap-2 flex-shrink-0">
                <!-- Loading Skeletons for Right Section -->
                <template v-if="!isReady">
                    <!-- Master EQ/Spectrum/FX Skeleton -->
                    <div class="flex flex-col h-full gap-2">
                        <div class="w-[36rem] flex flex-col flex-1 min-h-0 gap-2">
                            <div class="flex-1 min-h-0 bg-gray-800 rounded-lg border border-gray-700 animate-pulse">
                            </div>
                            <div class="flex-1 min-h-0 bg-gray-800 rounded-lg border border-gray-700 animate-pulse">
                            </div>
                        </div>
                        <div class="w-[36rem] bg-gray-800 rounded-lg border border-gray-700 p-4 animate-pulse"
                            style="height: 200px;"></div>
                    </div>

                    <!-- Subgroups Skeleton -->
                    <div class="w-32 h-full">
                        <div class="bg-gray-800 rounded-lg border border-gray-700 h-full p-2 flex flex-col gap-2">
                            <div class="h-6 bg-gray-700 rounded animate-pulse"></div>
                            <div class="flex-1 flex items-center justify-center py-4">
                                <div class="flex gap-2">
                                    <div class="w-5 flex-1 bg-gray-700 rounded animate-pulse"></div>
                                    <div class="w-5 flex-1 bg-gray-700 rounded animate-pulse"></div>
                                </div>
                            </div>
                            <div class="h-8 bg-gray-700 rounded animate-pulse"></div>
                        </div>
                    </div>

                    <!-- Master Skeleton -->
                    <div class="w-44 h-full">
                        <div class="bg-gray-800 rounded-lg border border-gray-700 h-full p-2 flex flex-col gap-2">
                            <div class="h-6 bg-gray-700 rounded animate-pulse"></div>
                            <div class="flex-1 flex items-center justify-center py-4">
                                <div class="flex gap-2">
                                    <div class="w-8 flex-1 bg-gray-700 rounded animate-pulse"></div>
                                    <div class="w-8 flex-1 bg-gray-700 rounded animate-pulse"></div>
                                </div>
                            </div>
                            <div class="h-8 bg-gray-700 rounded animate-pulse"></div>
                        </div>
                    </div>
                </template>

                <!-- Master EQ Display, Spectrum & FX (Draggable) -->
                <template v-else>
                    <RightSection ref="rightSectionRef" :master-channel="masterChannel"
                        :master-section-ref="masterSectionRef" :master-eq-output-node="masterEqOutputNode"
                        :master-fx-output-node="masterFxOutputNode" :aux-buses="auxBuses"
                        @master-eq-output-node="handleMasterEqOutputNode"
                        @master-fx-output-node="handleMasterFxOutputNode" @master-fx-component="handleMasterFxComponent"
                        @update:master-eq-filters="handleMasterEQFiltersUpdate" @add-aux="addAux"
                        @remove-aux="removeAux" @update-aux="updateAux" />

                    <!-- Subgroups Section -->
                    <template v-for="subgroup in subgroups" :key="subgroup.id">
                        <div class="flex-shrink-0 h-full mixer-fade-in">
                            <SubgroupsSection :ref="el => setSubgroupRef(subgroup.id, el)"
                                :master-channel="masterChannel" :subgroup-id="subgroup.id"
                                :subgroup-name="subgroup.name" @remove="removeSubgroup(subgroup.id)" />
                        </div>
                    </template>

                    <!-- Master Section -->
                    <div class="flex-shrink-0 h-full mixer-fade-in">
                        <MasterSection ref="masterSectionRef" :master-fx-output-node="masterFxOutputNode"
                            :master-fx-component="masterFxComponent" :loaded-tracks="loadedTracks" />
                    </div>
                </template>
            </div>
        </main>

        <!-- Footer Info -->
        <footer class="bg-black/50 backdrop-blur-sm border-t border-gray-700 px-6 py-2 fixed bottom-0 left-0 right-0 z-[100]">
            <div class="flex justify-between items-center text-xs text-gray-500">
                <div>
                    Built with Nuxt 3, Tone.js & Tailwind CSS
                </div>
                <div>
                    Sample Rate: {{ sampleRate }}Hz | Buffer Size: {{ bufferSize }}
                </div>
            </div>
        </footer>

        <!-- Automation Section (Resizable/Collapsible) - Fixed position overlay -->
        <div v-if="isReady" 
            class="automation-section fixed left-0 right-0 transition-all duration-300 ease-out border-t border-gray-700 z-[90] bg-gray-900"
            :style="{ height: automationCollapsed ? '17px' : automationHeight + 'px', bottom: '34px' }">
            
            <!-- Resize Handle -->
            <div 
                :class="['absolute left-0 right-0 top-0 h-4 z-10 group bg-gray-900/20', automationCollapsed ? 'cursor-default' : 'cursor-ns-resize']"
                @mousedown.stop="startAutomationResize"
                :title="automationCollapsed ? 'Panel collapsed' : 'Drag to resize'"
            >
                <!-- Collapse/Expand Button -->
                <button
                    @click.stop="toggleAutomationCollapse"
                    class="absolute -mt-2 left-2 top-1/2 -translate-y-1/2 w-5 h-5 bg-gray-800 hover:bg-blue-600 border border-gray-700 hover:border-blue-500 rounded flex items-center justify-center transition-all shadow-lg"
                    :title="automationCollapsed ? 'Expand automation panel' : 'Collapse automation panel'"
                >
                    <svg class="w-3 h-3 text-gray-300 transition-transform" :class="{ 'rotate-180': automationCollapsed }" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
                    </svg>
                </button>
                
                <!-- Grip Dots -->
                <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex flex-col gap-0.5 opacity-50 group-hover:opacity-100 transition-opacity pointer-events-none">
                    <!-- Row 1 -->
                    <div class="flex gap-0.5">
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                    </div>
                    <!-- Row 2 -->
                    <div class="flex gap-0.5">
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                        <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
                    </div>
                </div>
            </div>

            <!-- Automation Content (hidden when collapsed) -->
            <div v-show="!automationCollapsed" class="h-full overflow-hidden pt-[1rem]">
                <!-- Automation Timeline -->
                <Timeline 
                    :transport="automation.transport.value" 
                    :playhead-position="automation.playheadPosition.value"
                    :is-recording="automation.isRecording.value"
                    :loop-enabled="loopEnabled"
                    @play="handlePlay"
                    @pause="handlePause"
                    @stop="handleStop"
                    @record="toggleRecordMode"
                    @seek="automation.seek"
                    @update-bpm="(bpm) => automation.transport.value.bpm = bpm"
                    @update-time-signature="(num, den) => { automation.transport.value.timeSignature = { numerator: num, denominator: den } }"
                    @toggle-loop="loopEnabled = !loopEnabled"
                />

                <!-- Automation Lanes -->
                <div v-if="automation.automationLanes.value.length > 0" class="automation-lanes-container bg-gray-950 border-t border-gray-700 overflow-y-auto" 
                    :style="{ maxHeight: (automationHeight - 100) + 'px' }">
                    <AutomationLane
                        v-for="lane in automation.automationLanes.value"
                        :key="`${lane.trackId}-${lane.parameter}-${lane.auxId || ''}`"
                        :lane="lane"
                        :duration="automation.transport.value.duration"
                        :playhead-position="automation.playheadPosition.value"
                        :min-value="lane.parameter === 'volume' ? -60 : -1"
                        :max-value="lane.parameter === 'volume' ? 12 : 1"
                        :label="`Track ${lane.trackId} - ${lane.parameter.toUpperCase()}`"
                        @add-point="(time: number, value: number) => automation.addPoint(lane.trackId, lane.parameter, time, value, lane.auxId)"
                        @remove-point="(index: number) => automation.removePoint(lane.trackId, lane.parameter, index, lane.auxId)"
                        @update-point="(index: number, time: number, value: number) => automation.updatePoint(lane.trackId, lane.parameter, index, time, value, lane.auxId)"
                        @change-mode="(mode: any) => lane.mode = mode"
                    />
                </div>
            </div>
        </div>

        <!-- Audio Flow Modal -->
        <AudioFlowModal v-model="showAudioFlowModal" :subgroups="subgroups.map(s => ({ id: s.id, name: s.name }))"
            :aux-buses="auxBuses.map(a => ({ id: a.id, name: a.name }))" />

        <!-- File Manager Modal -->
        <FileManagerModal 
          v-model="showFileManager" 
          @select-file="handleFileManagerSelect"
          @select-playlist="handlePlaylistSelect" />

        <!-- Scenes Modal -->
        <ScenesModal v-model="showScenesModal" :scenes="scenes" :current-scene-id="currentSceneId"
            @save="handleSaveScene" @load="handleLoadScene" @update="handleUpdateScene" @delete="handleDeleteScene"
            @rename="handleRenameScene" @toggle-pin="handleTogglePin" />

        <!-- Limit Reached Modal -->
        <Transition enter-from-class="opacity-0" enter-active-class="transition-opacity duration-200"
            enter-to-class="opacity-100" leave-from-class="opacity-100" leave-active-class="transition-opacity duration-200"
            leave-to-class="opacity-0">
            <div v-if="showLimitModal" @click="showLimitModal = false"
                class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[9999] p-4">
                <div @click.stop
                    class="bg-gradient-to-br from-gray-800 to-gray-900 border-2 border-orange-500/70 rounded-lg shadow-2xl max-w-md w-full p-6">
                    <div class="flex items-start gap-3 mb-4">
                        <div class="flex-shrink-0 w-10 h-10 rounded-full bg-orange-500/20 flex items-center justify-center">
                            <svg class="w-6 h-6 text-orange-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                            </svg>
                        </div>
                        <div class="flex-1">
                            <h3 class="text-lg font-bold text-orange-400 mb-2">Limit Reached</h3>
                            <p class="text-gray-300 text-sm leading-relaxed" v-html="limitModalMessage"></p>
                        </div>
                    </div>
                    <div class="flex gap-3 mt-6">
                        <a href="https://www.mmpro.it" target="_blank"
                            class="flex-1 px-4 py-2.5 bg-gradient-to-r from-blue-600 to-blue-500 hover:from-blue-500 hover:to-blue-400 rounded-lg text-white font-semibold text-sm transition-all text-center">
                            Download Full Version
                        </a>
                        <button @click="showLimitModal = false"
                            class="px-4 py-2.5 bg-gray-700 hover:bg-gray-600 rounded-lg text-gray-300 font-semibold text-sm transition-all">
                            Close
                        </button>
                    </div>
                </div>
            </div>
        </Transition>

        <!-- Scene Loading Overlay -->
        <Transition enter-from-class="opacity-0 scale-90 -translate-y-12"
            enter-active-class="transition-all duration-500 ease-out"
            enter-to-class="opacity-100 scale-100 translate-y-0" leave-from-class="opacity-100 scale-100 translate-y-0"
            leave-active-class="transition-all duration-300 ease-in" leave-to-class="opacity-0 scale-90 -translate-y-8">
            <div v-if="isLoadingScene" class="fixed inset-0 flex items-start justify-center pt-20 z-[9999]">
                <div
                    class="bg-gradient-to-br from-gray-600 to-gray-700 border-2 border-blue-500/70 rounded-lg shadow-2xl px-6 py-3 flex items-center gap-3 whitespace-nowrap">
                    <div class="relative w-5 h-5">
                        <div class="absolute inset-0 border-2 border-blue-500/30 rounded-full"></div>
                        <div
                            class="absolute inset-0 border-2 border-transparent border-t-blue-500 rounded-full animate-spin">
                        </div>
                    </div>
                    <span class="text-sm font-semibold text-white">Loading Scene</span>
                    <div class="flex gap-1 pt-2">
                        <div class="w-1.5 h-1.5 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 0ms">
                        </div>
                        <div class="w-1.5 h-1.5 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 150ms">
                        </div>
                        <div class="w-1.5 h-1.5 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 300ms">
                        </div>
                    </div>
                </div>
            </div>
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, toRaw, nextTick, inject, watch, onUnmounted, provide, type Ref } from 'vue'
import AudioTrack from './components/AudioTrack.vue'
import SignalTrack from './components/SignalTrack.vue'
import AudioFlowModal from './components/layout/AudioFlowModal.vue'
import FileManagerModal from './components/layout/FileManagerModal.vue'
import RightSection from './components/master/RightSection.vue'
import MasterSection from './components/MasterSection.vue'
import SubgroupsSection from './components/SubgroupsSection.vue'
import ScenesModal from './components/layout/ScenesModal.vue'
import Timeline from './components/automation/Timeline.vue'
import AutomationLane from './components/automation/AutomationLane.vue'
import { useAudioDevices } from '~/composables/useAudioDevices'
import { useScenes, type Scene, type TrackSnapshot, type SubgroupSnapshot, type AuxSnapshot } from '~/composables/useScenes'
import { useAudioFileStorage } from '~/composables/useAudioFileStorage'
import { useAutomation } from '~/composables/useAutomation'
import { getBuildLimits, canAddTrack, getTrackCounts, getBuildMode } from '~/config/buildLimits'
import { channel } from 'diagnostics_channel'

const ToneRef = inject<any>('Tone')
let Tone: any = null
const toneReady = ref(false)
const masterChannel = ref<any>(null)

// Subgroups system
interface Subgroup {
    id: number
    name: string
    channel: any
    ref: any
}

const subgroups = ref<Subgroup[]>([])
let nextSubgroupId = 1

// Build limits
const buildLimits = computed(() => getBuildLimits())
const buildMode = computed(() => getBuildMode())

// Aux buses system
interface AuxBus {
    id: string
    name: string
    volume: number
    muted: boolean
    soloed: boolean
    routeToMaster: boolean
    selectedOutputDevice?: string | null
    node?: any  // Input node (Channel)
    outputNode?: any  // Output node (final node of FX chain)
    outputStreamDest?: MediaStreamAudioDestinationNode | null
    outputAudioContext?: AudioContext | null
    outputSource?: MediaStreamAudioSourceNode | null
    // FX Chain
    reverbNode?: any
    reverbEnabled?: boolean
    reverbParams?: { decay: number, preDelay: number, wet: number }
    delayNode?: any
    delayEnabled?: boolean
    delayParams?: { delayTime: number, feedback: number, wet: number }
}

const auxBuses = ref<AuxBus[]>([])
let nextAuxId = 1

interface Track {
    id: number
    type: 'audio' | 'signal'
    order: number
}

// App ready state
const isReady = ref(false)
const isAppReady = inject<Ref<boolean>>('isAppReady', ref(false))

// Audio Flow Modal
const showAudioFlowModal = ref(false)
const showScenesModal = ref(false)
const showFileManager = ref(false)
const isLoadingScene = ref(false)
const showLimitModal = ref(false)
const limitModalMessage = ref('')

// Automation System
const automation = useAutomation()
const loopEnabled = ref(false)
const playbackLoopId = ref<number | null>(null)
const armedTracks = ref<Set<number>>(new Set()) // Tracks armed for recording

// Automation panel resize/collapse
const automationHeight = ref(220) // Default height in pixels
const automationCollapsed = ref(true)
const automationSavedHeight = ref(300)
let automationResizing = false
let automationStartY = 0
let automationStartHeight = 0
let automationResizeRafId: number | null = null
let automationPendingHeight: number | null = null

// Provide automation to child components
provide('automation', automation)

// File Manager for tracks (Electron only)
const fileManagerTargetTrackId = ref<number | null>(null)

function openFileManagerForTrack(trackId: number) {
    fileManagerTargetTrackId.value = trackId
    showFileManager.value = true
}

function handleFileManagerSelect(file: any) {
    let targetTrackId = fileManagerTargetTrackId.value
    
    // If no specific track was selected (opened from top bar), find first free audio track
    if (targetTrackId === null) {
        targetTrackId = findFirstFreeAudioTrack()
        if (targetTrackId === null) {
            alert('No free audio tracks available. All tracks have files loaded.')
            showFileManager.value = false
            return
        }
    }
    
    const trackRef = trackRefs.value.get(targetTrackId)
    if (trackRef && trackRef.loadFileFromLibrary) {
        trackRef.loadFileFromLibrary(file)
    }
    
    fileManagerTargetTrackId.value = null
    showFileManager.value = false
}

function handlePlaylistSelect(playlist: any) {
    let targetTrackId = fileManagerTargetTrackId.value
    
    // If no specific track was selected (opened from top bar), find first free audio track
    if (targetTrackId === null) {
        targetTrackId = findFirstFreeAudioTrack()
        if (targetTrackId === null) {
            alert('No free audio tracks available. All tracks have files loaded.')
            showFileManager.value = false
            return
        }
    }
    
    const trackRef = trackRefs.value.get(targetTrackId)
    if (trackRef && trackRef.loadPlaylistFromLibrary) {
        trackRef.loadPlaylistFromLibrary(playlist)
    }
    
    fileManagerTargetTrackId.value = null
    showFileManager.value = false
}

// Find first audio track without a file loaded
function findFirstFreeAudioTrack(): number | null {
    for (const track of sortedTracks.value) {
        if (track.type === 'audio') {
            const trackRef = trackRefs.value.get(track.id)
            if (trackRef && trackRef.isAudioLoaded && !trackRef.isAudioLoaded()) {
                return track.id
            }
        }
    }
    return null
}

// Provide file manager API to child components
provide('fileManager', {
    openFileManager: openFileManagerForTrack
})

// Tracks management
// Initialize with tracks based on build mode
function initializeTracks(): Track[] {
    const limits = getBuildLimits()
    const tracks: Track[] = []
    let id = 1
    let order = 1
    
    // Add audio tracks
    for (let i = 0; i < limits.defaultAudioTracks; i++) {
        tracks.push({ id: id++, type: 'audio', order: order++ })
    }
    
    // Add signal tracks
    for (let i = 0; i < limits.defaultSignalTracks; i++) {
        tracks.push({ id: id++, type: 'signal', order: order++ })
    }
    
    return tracks
}

const tracks = ref<Track[]>(initializeTracks())

// Computed per ordinare le tracce per order
const sortedTracks = computed(() => {
    return [...tracks.value].sort((a, b) => a.order - b.order)
})

const showAddTrackMenu = ref(false)

function getNextAvailableId(): number {
    // Find the smallest available ID from 1 to 24
    for (let i = 1; i <= 24; i++) {
        if (!tracks.value.find(t => t.id === i)) {
            return i
        }
    }
    // If all 1-24 are taken, return the next number
    return Math.max(...tracks.value.map(t => t.id)) + 1
}

function handleAddButtonClick() {
    // Check if we've reached the total track limit
    if (tracks.value.length >= buildLimits.value.maxTracks) {
        const limits = buildLimits.value
        const mode = buildMode.value
        limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxTracks} total tracks</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for unlimited tracks.`
        showLimitModal.value = true
        return
    }
    
    // Check if we can add at least one type of track
    const canAddAudio = canAddTrack(tracks.value, 'audio')
    const canAddSignal = canAddTrack(tracks.value, 'signal')
    
    if (!canAddAudio && !canAddSignal) {
        // Can't add any type of track
        const limits = buildLimits.value
        const mode = buildMode.value
        limitModalMessage.value = `You've reached the limits for all track types in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for unlimited tracks.`
        showLimitModal.value = true
        return
    }
    
    // Open the menu
    showAddTrackMenu.value = !showAddTrackMenu.value
}

function addTrackOfType(type: 'audio' | 'signal') {
    // Check if we can add this track type
    if (!canAddTrack(tracks.value, type)) {
        const limits = buildLimits.value
        const counts = getTrackCounts(tracks.value)
        const mode = buildMode.value
        
        if (type === 'audio' && counts.audio >= limits.maxAudioTracks) {
            limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxAudioTracks} audio track${limits.maxAudioTracks > 1 ? 's' : ''}</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for unlimited tracks.`
        } else if (type === 'signal' && counts.signal >= limits.maxSignalTracks) {
            limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxSignalTracks} signal track${limits.maxSignalTracks > 1 ? 's' : ''}</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for unlimited tracks.`
        } else {
            limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxTracks} total tracks</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for unlimited tracks.`
        }
        showLimitModal.value = true
        showAddTrackMenu.value = false
        return
    }
    
    const newId = getNextAvailableId()
    const maxOrder = tracks.value.length > 0 ? Math.max(...tracks.value.map(t => t.order)) : 0
    tracks.value.push({ id: newId, type, order: maxOrder + 1 })
    showAddTrackMenu.value = false
}

function removeTrack(trackId: number) {
    if (tracks.value.length <= 1) return

    const trackIndex = tracks.value.findIndex(t => t.id === trackId)
    if (trackIndex === -1) return

    // Ask for confirmation
    const trackType = tracks.value[trackIndex].type === 'audio' ? 'Audio Track' : 'Signal Track'
    if (!confirm(`Remove ${trackType} ${trackId}?`)) {
        return
    }

    const removedTrack = tracks.value.splice(trackIndex, 1)[0]

    // Remove the track ref from the map
    if (removedTrack) {
        trackRefs.value.delete(removedTrack.id)
        // Also remove from solo tracks if it was soloed
        soloTracks.value.delete(removedTrack.id)
    }
}

// Drag and Drop for track reordering
const draggedTrackId = ref<number | null>(null)
const dragOverTrackId = ref<number | null>(null)

function handleTrackDragStart(trackId: number) {
    draggedTrackId.value = trackId
}

function handleTrackDragOver(trackId: number, event: DragEvent) {
    event.preventDefault() // Necessary to allow drop
    if (draggedTrackId.value === trackId) return
    dragOverTrackId.value = trackId
}

function handleTrackDragLeave() {
    dragOverTrackId.value = null
}

function handleTrackDrop(targetTrackId: number) {
    const draggedId = draggedTrackId.value
    if (draggedId === null || draggedId === targetTrackId) {
        draggedTrackId.value = null
        dragOverTrackId.value = null
        return
    }

    // Find tracks
    const draggedTrack = tracks.value.find(t => t.id === draggedId)
    const targetTrack = tracks.value.find(t => t.id === targetTrackId)

    if (!draggedTrack || !targetTrack) {
        draggedTrackId.value = null
        dragOverTrackId.value = null
        return
    }

    // Scambia gli order delle due tracce
    const tempOrder = draggedTrack.order
    draggedTrack.order = targetTrack.order
    targetTrack.order = tempOrder

    // Clear drag state
    draggedTrackId.value = null
    dragOverTrackId.value = null
}

function handleTrackDragEnd() {
    draggedTrackId.value = null
    dragOverTrackId.value = null
}

// Track refs management (only for tracks, not for master components)
const trackRefs = ref<Map<number, any>>(new Map())
const masterSectionRef = ref<any>(null) // Keep only for getSnapshot/restoreSnapshot
const rightSectionRef = ref<any>(null) // Ref to RightSection component

// Audio nodes received from components via emit
const masterEqOutputNode = ref<any>(null)
const masterFxOutputNode = ref<any>(null)
const masterFxComponent = ref<any>(null) // For getSnapshot only

// Handlers for output node updates
function handleMasterEqOutputNode(node: any) {
    masterEqOutputNode.value = node
}

function handleMasterFxOutputNode(node: any) {
    masterFxOutputNode.value = node
}

function handleMasterFxComponent(component: any) {
    masterFxComponent.value = component
}

// Handle master EQ filters update from RightSection
function handleMasterEQFiltersUpdate(filters: any[]) {
    // Update is handled internally by RightSection
    // This handler is kept for potential future use
}

function setTrackRef(trackId: number, el: any | null) {
    if (el) {
        trackRefs.value.set(trackId, el)
    } else {
        // Remove ref when component is unmounted
        trackRefs.value.delete(trackId)
    }
}

// Subgroup management
function setSubgroupRef(subgroupId: number, el: any | null) {
    const subgroup = subgroups.value.find(s => s.id === subgroupId)
    if (subgroup && el) {
        subgroup.ref = el

        // Connect channel to subgroup input when ref is set
        nextTick(() => {
            if (subgroup.channel && el.getInputNode) {
                const inputNode = el.getInputNode()
                if (inputNode) {
                    const rawChannel = toRaw(subgroup.channel)
                    const rawInputNode = toRaw(inputNode)
                    try {
                        rawChannel.connect(rawInputNode)
                    } catch (e) {
                        console.error(`[Subgroup ${subgroup.name}] Connection error:`, e)
                    }
                }
            }
        })
    }
}

function addSubgroup() {
    if (!Tone) return

    // Check build limits
    if (subgroups.value.length >= buildLimits.value.maxSubgroups) {
        const limits = buildLimits.value
        const mode = buildMode.value
        limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxSubgroups} subgroup${limits.maxSubgroups > 1 ? 's' : ''}</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for more subgroups.`
        showLimitModal.value = true
        return
    }

    const id = nextSubgroupId++
    const name = `SUB ${id}`

    // Create Tone.js channel for this subgroup
    const channel = new Tone.Channel({
        volume: 0,
        pan: 0,
        channelCount: 2,
        channelCountMode: 'explicit',
        channelInterpretation: 'speakers'
    })

    subgroups.value.push({
        id,
        name,
        channel,
        ref: null
    })
}

function removeSubgroup(subgroupId: number) {
    const index = subgroups.value.findIndex(s => s.id === subgroupId)
    if (index !== -1) {
        const subgroup = subgroups.value[index]

        // Ask for confirmation
        if (!confirm(`Remove ${subgroup.name}?`)) {
            return
        }

        // Disconnect all tracks from this subgroup
        loadedTracks.value.forEach(track => {
            const trackRef = trackRefs.value.get(track.trackNumber)
            if (trackRef?.disconnectFromSubgroup) {
                trackRef.disconnectFromSubgroup(subgroupId)
            }
        })

        // Remove from array - Vue will handle unmounting and cleanup via onUnmounted
        subgroups.value.splice(index, 1)
    }
}

// Aux buses management
function addAux() {
    if (!Tone) return

    // Check build limits
    if (auxBuses.value.length >= buildLimits.value.maxAuxBuses) {
        const limits = buildLimits.value
        const mode = buildMode.value
        limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxAuxBuses} aux bus${limits.maxAuxBuses > 1 ? 'es' : ''}</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for more aux buses.`
        showLimitModal.value = true
        return
    }

    const id = `aux-${nextAuxId++}`
    const name = `AUX ${nextAuxId - 1}`

    // Create Tone.js channel for this aux - this receives sends from tracks
    const inputNode = new Tone.Channel({
        volume: 0,
        pan: 0,
        mute: false,
        channelCount: 2,
        channelCountMode: 'explicit',
        channelInterpretation: 'speakers'
    })

    // Create FX chain
    // Reverb (initially disabled - wet will be set to 1.0 when enabled)
    const reverbNode = new Tone.Reverb({
        decay: 2.5,
        preDelay: 0.01,
        wet: 0  // Start disabled
    })
    
    // Generate reverb impulse response (required for Tone.Reverb)
    reverbNode.generate().then(() => {
    }).catch((err: any) => {
        console.error(`[AUX ${name}] Reverb generation failed:`, err)
    })

    // Delay (initially disabled - wet will be set to 1.0 when enabled)
    const delayNode = new Tone.FeedbackDelay({
        delayTime: 0.25,  // 250ms (quarter note at 120bpm)
        feedback: 0.3,
        wet: 0  // Start disabled
    })
    
    // Output node (final point of FX chain)
    const outputNode = new Tone.Gain(1)

    // FX Chain: input → reverb → delay → outputNode → destinations
    // outputNode is the final node that connects to master and/or output device
    
    // Create MediaStreamDestination for output routing
    const mainAudioContext = Tone.context.rawContext as AudioContext
    const outputStreamDest = mainAudioContext.createMediaStreamDestination()

    // Connect FX chain - ALWAYS connected, bypass via wet=0
    inputNode.connect(reverbNode)
    reverbNode.connect(delayNode)
    delayNode.connect(outputNode)
    
    // Output node connects to stream destination by default
    outputNode.connect(outputStreamDest as any)

    const newAux: AuxBus = {
        id,
        name,
        volume: 0,
        muted: false,
        soloed: false,
        routeToMaster: false,
        selectedOutputDevice: 'no-output',
        node: inputNode,
        outputNode,
        outputStreamDest,
        outputAudioContext: null,
        outputSource: null,
        // FX
        reverbNode,
        reverbEnabled: false,
        reverbParams: { decay: 2.5, preDelay: 0.01, wet: 1.0 },
        delayNode,
        delayEnabled: false,
        delayParams: { delayTime: 0.25, feedback: 0.3, wet: 1.0 }
    }

    auxBuses.value.push(newAux)
}

function removeAux(index: number) {
    if (index >= 0 && index < auxBuses.value.length) {
        const aux = auxBuses.value[index]

        // Ask for confirmation
        if (!confirm(`Remove ${aux.name}?`)) {
            return
        }

        // Disconnect and close output
        if (aux.outputSource) {
            try {
                aux.outputSource.disconnect()
            } catch (e) { }
        }

        if (aux.outputAudioContext) {
            aux.outputAudioContext.close()
        }

        // Dispose Tone.js node
        if (aux.node) {
            aux.node.dispose()
        }

        // Remove from array
        auxBuses.value.splice(index, 1)
    }
}

async function updateAux(index: number, updatedAux: AuxBus) {
    if (index >= 0 && index < auxBuses.value.length) {
        const aux = auxBuses.value[index]

        // Update routing to master if changed
        if (updatedAux.routeToMaster !== aux.routeToMaster) {
            const outputNode = toRaw(aux.outputNode)  // Use outputNode (end of FX chain)
            const masterChan = toRaw(masterChannel.value)
            const outputStreamDest = toRaw(aux.outputStreamDest)

            if (updatedAux.routeToMaster && masterChan) {
                try {
                    outputNode.disconnect()
                } catch (e) { }
                // Connect to both master and stream destination
                outputNode.connect(masterChan)
                if (outputStreamDest) {
                    outputNode.connect(outputStreamDest as any)
                }
            } else {
                try {
                    outputNode.disconnect()
                } catch (e) { }
                // Connect only to stream destination
                if (outputStreamDest) {
                    outputNode.connect(outputStreamDest as any)
                }
            }
        }

        // Handle output device change BEFORE updating values
        if (updatedAux.selectedOutputDevice !== aux.selectedOutputDevice) {
            await changeAuxOutputDevice(index, updatedAux.selectedOutputDevice)
        }

        // Update values (preserve audio nodes and output routing objects that are managed separately)
        auxBuses.value[index] = {
            ...updatedAux,
            node: auxBuses.value[index].node,
            outputNode: auxBuses.value[index].outputNode,
            reverbNode: auxBuses.value[index].reverbNode,
            delayNode: auxBuses.value[index].delayNode,
            outputStreamDest: auxBuses.value[index].outputStreamDest,
            outputAudioContext: auxBuses.value[index].outputAudioContext,
            outputSource: auxBuses.value[index].outputSource
        }
    }
}

// Change aux output device
async function changeAuxOutputDevice(index: number, deviceId: string | null | undefined) {
    if (index < 0 || index >= auxBuses.value.length) return

    const aux = auxBuses.value[index]
    if (!aux.outputStreamDest || !Tone) return

    try {
        // Disconnect and close existing output
        if (aux.outputSource) {
            console.log(`[Aux ${aux.name}] Disconnecting old source`)
            try {
                aux.outputSource.disconnect()
            } catch (e) {
                console.warn(`[Aux ${aux.name}] Error disconnecting source:`, e)
            }
            auxBuses.value[index].outputSource = null
        }

        if (aux.outputAudioContext) {
            console.log(`[Aux ${aux.name}] Closing old AudioContext, state:`, aux.outputAudioContext.state)
            try {
                if (aux.outputAudioContext.state !== 'closed') {
                    await aux.outputAudioContext.close()
                }
            } catch (e) {
                console.warn(`[Aux ${aux.name}] Error closing context:`, e)
            }
            auxBuses.value[index].outputAudioContext = null
        }

        // Small delay to ensure cleanup is complete
        await new Promise(resolve => setTimeout(resolve, 50))

        // If "no-output" is selected, don't create any output context
        if (deviceId === 'no-output') {
            return
        }

        // Parse composite deviceId (format: "realDeviceId:channelIndex")
        let realDeviceId = deviceId || ''
        let targetChannel: number | null = null
        
        if (deviceId && deviceId.includes(':')) {
            const parts = deviceId.split(':')
            realDeviceId = parts[0]
            targetChannel = parseInt(parts[1], 10)
            console.log(`[Aux ${aux.name}] Parsed composite deviceId: device="${realDeviceId}", channel=${targetChannel + 1}`)
        }

        // Create new AudioContext targeting selected device
        const mainAudioContext = Tone.context.rawContext as AudioContext
        const contextOptions: any = {
            latencyHint: 'interactive',
            sampleRate: mainAudioContext.sampleRate
        }

        if (realDeviceId && realDeviceId !== '') {
            contextOptions.sinkId = realDeviceId
        }

        const outputAudioContext = new AudioContext(contextOptions)
        
        // Log device info
        console.log(`[Aux ${aux.name}] Output AudioContext created`)
        console.log(`[Aux ${aux.name}] Destination maxChannelCount:`, outputAudioContext.destination.maxChannelCount)
        console.log(`[Aux ${aux.name}] SinkId:`, (outputAudioContext as any).sinkId)
        
        // Detect number of output channels from device capabilities
        let deviceChannelCount = outputAudioContext.destination.maxChannelCount
        
        // If we have a target channel from composite ID, use that as indicator of multi-channel device
        if (targetChannel !== null) {
            // Target channel tells us the device has at least targetChannel+1 channels
            // For Rubix44 we know it has 4 channels
            deviceChannelCount = Math.max(4, targetChannel + 1)
        }
        
        console.log(`[Aux ${aux.name}] Device channel count: ${deviceChannelCount}`)
        
        // Configure destination for multi-channel output
        try {
            outputAudioContext.destination.channelCount = deviceChannelCount
            outputAudioContext.destination.channelCountMode = 'explicit'
            outputAudioContext.destination.channelInterpretation = 'discrete'
            console.log(`[Aux ${aux.name}] Set destination to ${deviceChannelCount} channels (discrete)`)
        } catch (e) {
            console.warn(`[Aux ${aux.name}] Could not configure destination:`, e)
        }
        
        // Create audio routing
        const source = outputAudioContext.createMediaStreamSource(aux.outputStreamDest.stream)
        
        // Check actual channel count from the source
        const actualChannelCount = source.channelCount
        console.log(`[Aux ${aux.name}] Source has ${actualChannelCount} channels`)
        
        // If a specific channel was selected (from composite deviceId), route to that channel
        if (targetChannel !== null && deviceChannelCount > 2) {
            // Create a channel merger to route aux to specific output channels
            const channelMerger = outputAudioContext.createChannelMerger(deviceChannelCount)
            
            if (actualChannelCount === 2) {
                // Stereo source - split and route to consecutive channels
                const splitter = outputAudioContext.createChannelSplitter(2)
                source.connect(splitter)
                
                // Route left to target channel, right to target+1 (if stereo width allows)
                splitter.connect(channelMerger, 0, targetChannel)
                if (targetChannel + 1 < deviceChannelCount) {
                    splitter.connect(channelMerger, 1, targetChannel + 1)
                    console.log(`[Aux ${aux.name}] Routing stereo to output channels ${targetChannel + 1}-${targetChannel + 2} of ${deviceChannelCount}`)
                } else {
                    console.log(`[Aux ${aux.name}] Routing mono (left) to output channel ${targetChannel + 1} of ${deviceChannelCount}`)
                }
            } else {
                // Mono source - route directly to target channel
                const monoGain = outputAudioContext.createGain()
                source.connect(monoGain)
                monoGain.connect(channelMerger, 0, targetChannel)
                console.log(`[Aux ${aux.name}] Routing mono to output channel ${targetChannel + 1} of ${deviceChannelCount}`)
            }
            
            // Connect merger to destination
            channelMerger.connect(outputAudioContext.destination)
        } else {
            // Default routing (stereo output or no specific channel selected)
            source.connect(outputAudioContext.destination)
            console.log(`[Aux ${aux.name}] Default stereo routing`)
        }

        // Resume if suspended
        if (outputAudioContext.state === 'suspended') {
            await outputAudioContext.resume()
        }

        // Store the new context and source
        auxBuses.value[index].outputAudioContext = outputAudioContext
        auxBuses.value[index].outputSource = source
    } catch (error) {
        console.error(`[Aux ${aux.name}] Error changing output device:`, error)
    }
}

// Loaded tracks for recorder waveform display
const loadedTracks = computed(() => {
    const tracks: Array<{
        trackNumber: number,
        fileName: string,
        fileId: string,
        isPlaying: boolean,
        currentTime: number,
        duration: number
    }> = []
    trackRefs.value.forEach((trackRef, trackId) => {
        if (trackRef && typeof trackRef.getSnapshot === 'function') {
            const snapshot = trackRef.getSnapshot()
            // Only show tracks that are playing AND are file-based
            if (snapshot.sourceType === 'file' && snapshot.fileName && snapshot.fileId && snapshot.isPlaying) {
                tracks.push({
                    trackNumber: snapshot.trackNumber,
                    fileName: snapshot.fileName,
                    fileId: snapshot.fileId,
                    isPlaying: snapshot.isPlaying,
                    currentTime: snapshot.currentTime || 0,
                    duration: snapshot.duration || 0
                })
            }
        }
    })
    return tracks.sort((a, b) => a.trackNumber - b.trackNumber)
})

// Solo handling
const soloTracks = ref<Set<number>>(new Set())

function handleSoloChange(data: { trackNumber: number, isSolo: boolean }) {
    if (data.isSolo) {
        soloTracks.value.add(data.trackNumber)
    } else {
        soloTracks.value.delete(data.trackNumber)
    }

    // Update all tracks based on solo state
    trackRefs.value.forEach((trackRef, trackId) => {
        if (soloTracks.value.size > 0) {
            // If any track is soloed, mute all others
            const shouldBeMuted = !soloTracks.value.has(trackId)
            trackRef.setMuted(shouldBeMuted && !trackRef.isSolo())
        } else {
            // If no tracks are soloed, unmute all
            trackRef.setMuted(false)
        }
    })
}

// Level updates (for future visualizations)
function handleLevelUpdate(data: { trackNumber: number, level: number }) {
    // Can be used for additional visualizations if needed
}

// Track arming for automation recording
function toggleTrackArm(trackId: number) {
    if (armedTracks.value.has(trackId)) {
        armedTracks.value.delete(trackId)
    } else {
        armedTracks.value.add(trackId)
    }
    // Trigger reactivity
    armedTracks.value = new Set(armedTracks.value)
}

function isTrackArmed(trackId: number): boolean {
    return armedTracks.value.has(trackId)
}

// Audio context info (reactive)
const sampleRate = computed(() => {
    if (toneReady.value && Tone?.context?.sampleRate) {
        return Math.round(Tone.context.sampleRate)
    }
    return 0
})
const bufferSize = computed(() => {
    if (toneReady.value && Tone?.context?.rawContext) {
        const audioContext = Tone.context.rawContext

        return audioContext.baseLatency
            ? Math.round(audioContext.baseLatency * audioContext.sampleRate)
            : (audioContext.sampleRate ? 128 : 0) // Default buffer size estimate
    }
    return 0
})

// Audio devices enumeration (shared across all tracks)
const { enumerateAudioInputs } = useAudioDevices()

// Scene management
const {
    scenes,
    currentSceneId,
    loadScenesFromStorage,
    createScene,
    updateScene,
    deleteScene,
    renameScene,
    setCurrentScene,
    togglePinScene
} = useScenes()

const { deleteAudioFile } = useAudioFileStorage()

// Helper function to restore subgroup snapshots
function restoreSubgroupSnapshots(subgroupSnapshots: SubgroupSnapshot[]) {
    subgroupSnapshots.forEach((subgroupSnapshot: SubgroupSnapshot) => {
        const subgroup = subgroups.value.find(s => s.id === subgroupSnapshot.id)
        if (subgroup && subgroup.ref && subgroup.ref.restoreSnapshot) {
            subgroup.ref.restoreSnapshot(subgroupSnapshot)
        } else {
            console.warn(`[Scene Load] Cannot restore subgroup ${subgroupSnapshot.name} (ID: ${subgroupSnapshot.id})`,
                { hasSubgroup: !!subgroup, hasRef: !!subgroup?.ref })
        }
    })
}

async function handleSaveScene(sceneName: string) {
    // Collect snapshots from all tracks
    const trackSnapshots: TrackSnapshot[] = []
    tracks.value.forEach(track => {
        const trackRef = trackRefs.value.get(track.id)
        if (trackRef && trackRef.getSnapshot) {
            trackSnapshots.push(trackRef.getSnapshot())
        }
    })

    // Collect master snapshot from MasterSection
    const masterSectionSnapshot = masterSectionRef.value?.getSnapshot?.() || {
        leftVolume: 0,
        rightVolume: 0,
        headphonesVolume: 0,
        isLinked: true,
        compressorEnabled: false,
        reverbEnabled: false,
        delayEnabled: false,
        limiterEnabled: false
    }

    const masterSnapshot = {
        ...masterSectionSnapshot,
        masterEQFilters: (rightSectionRef.value?.masterEqFiltersData || []).map((filter: any) => ({
            type: filter.type,
            frequency: filter.frequency,
            gain: filter.gain,
            Q: filter.Q,
            color: filter.color
        }))
    }

    // Collect subgroup snapshots
    const subgroupSnapshots: SubgroupSnapshot[] = []
    subgroups.value.forEach(subgroup => {
        if (subgroup.ref && subgroup.ref.getSnapshot) {
            const snapshot = subgroup.ref.getSnapshot()
            subgroupSnapshots.push({
                id: subgroup.id,
                name: subgroup.name,
                ...snapshot
            })
        } else {
            console.warn(`[Scene Save] Subgroup ${subgroup.name} (ID: ${subgroup.id}) has no ref or getSnapshot`)
        }
    })

    // Collect aux snapshots
    const auxSnapshots: AuxSnapshot[] = []
    auxBuses.value.forEach(aux => {
        auxSnapshots.push({
            id: aux.id,
            name: aux.name,
            volume: aux.volume,
            muted: aux.muted,
            soloed: aux.soloed,
            routeToMaster: aux.routeToMaster,
            selectedOutputDevice: aux.selectedOutputDevice,
            // FX
            reverbEnabled: aux.reverbEnabled,
            reverbParams: aux.reverbParams,
            delayEnabled: aux.delayEnabled,
            delayParams: aux.delayParams
        })
    })

    // Create and save scene
    const scene = await createScene(
        sceneName, 
        trackSnapshots, 
        masterSnapshot, 
        subgroupSnapshots, 
        auxSnapshots,
        automation.exportAutomation() // Add automation data
    )
    setCurrentScene(scene.id)
}

function handleLoadScene(sceneId: string) {
    const scene = scenes.value.find((s: Scene) => s.id === sceneId)
    if (!scene) return

    // Close the modal first
    showScenesModal.value = false

    // Show loading overlay
    isLoadingScene.value = true

    // Small delay before starting the animation
    setTimeout(() => {
        // Reset all tracks to defaults before loading scene
        trackRefs.value.forEach((trackRef) => {
            if (trackRef && trackRef.resetToDefaults) {
                trackRef.resetToDefaults()
            }
        })

        // Reset master section to defaults
        if (masterSectionRef.value?.resetToDefaults) {
            masterSectionRef.value.resetToDefaults()
        }

        // Reset subgroups to defaults
        subgroups.value.forEach(subgroup => {
            if (subgroup.ref && subgroup.ref.resetToDefaults) {
                subgroup.ref.resetToDefaults()
            }
        })

        // Clear master EQ filters
        if (rightSectionRef.value?.masterEqFiltersData) {
            rightSectionRef.value.masterEqFiltersData = []
        }

        // Animate faders to -∞ (mute) first (digital mixer effect)
        trackRefs.value.forEach((trackRef) => {
            if (trackRef && trackRef.getSnapshot) {
                const snapshot = trackRef.getSnapshot()
                // Create a temporary snapshot with volume at -90 (-∞ / muted)
                const muteSnapshot = { ...snapshot, volume: -90 }
                trackRef.restoreFromSnapshot(muteSnapshot)
            }
        })

        // Wait before restoring actual values (give time to see the animation)
        setTimeout(() => {
            // Restore aux buses FIRST (before tracks, so aux nodes exist when tracks restore their sends)
            if (scene.auxBuses) {
                // Clean up existing aux buses
                while (auxBuses.value.length > 0) {
                    const aux = auxBuses.value[0]
                    
                    // Disconnect Tone.js node first to avoid InvalidStateError
                    if (aux.node) {
                        try {
                            toRaw(aux.node).disconnect()
                        } catch (e) { 
                            console.warn('[Scene Load] Error disconnecting aux node:', e)
                        }
                    }
                    
                    // Disconnect and close output
                    if (aux.outputSource) {
                        try {
                            aux.outputSource.disconnect()
                        } catch (e) { }
                    }
                    if (aux.outputAudioContext) {
                        try {
                            aux.outputAudioContext.close()
                        } catch (e) { }
                    }
                    
                    // Dispose Tone.js node
                    if (aux.node) {
                        try {
                            aux.node.dispose()
                        } catch (e) {
                            console.warn('[Scene Load] Error disposing aux node:', e)
                        }
                    }
                    auxBuses.value.shift()
                }

                // Create aux buses from scene
                scene.auxBuses.forEach((auxSnapshot: AuxSnapshot) => {
                    // Create Tone.js channel for this aux (input node)
                    const node = new Tone.Channel({
                        volume: auxSnapshot.volume,
                        pan: 0,
                        mute: auxSnapshot.muted,
                        channelCount: 2,
                        channelCountMode: 'explicit',
                        channelInterpretation: 'speakers'
                    })

                    // Create FX chain with saved parameters
                    const reverbParams = auxSnapshot.reverbParams || { decay: 2.5, preDelay: 0.01, wet: 1.0 }
                    const reverbNode = new Tone.Reverb({
                        decay: reverbParams.decay,
                        preDelay: reverbParams.preDelay,
                        wet: auxSnapshot.reverbEnabled ? reverbParams.wet : 0
                    })
                    
                    reverbNode.generate().catch((err: any) => {
                        console.error(`[Aux ${auxSnapshot.name}] Reverb generation failed:`, err)
                    })
                    
                    const delayParams = auxSnapshot.delayParams || { delayTime: 0.25, feedback: 0.3, wet: 1.0 }
                    const delayNode = new Tone.FeedbackDelay({
                        delayTime: delayParams.delayTime,
                        feedback: delayParams.feedback,
                        wet: auxSnapshot.delayEnabled ? delayParams.wet : 0
                    })
                    
                    // Output node (final point of FX chain)
                    const outputNode = new Tone.Gain(1)

                    // Create MediaStreamDestination for output routing
                    const mainAudioContext = Tone.context.rawContext as AudioContext
                    const outputStreamDest = mainAudioContext.createMediaStreamDestination()

                    // Connect FX chain
                    node.connect(reverbNode)
                    reverbNode.connect(delayNode)
                    delayNode.connect(outputNode)
                    
                    // Connect outputNode based on routeToMaster setting
                    if (auxSnapshot.routeToMaster) {
                        const masterChan = toRaw(masterChannel.value)
                        if (masterChan) {
                            outputNode.connect(masterChan)
                        }
                        outputNode.connect(outputStreamDest as any)
                    } else {
                        outputNode.connect(outputStreamDest as any)
                    }

                    const newAux: AuxBus = {
                        id: auxSnapshot.id,
                        name: auxSnapshot.name,
                        volume: auxSnapshot.volume,
                        muted: auxSnapshot.muted,
                        soloed: auxSnapshot.soloed,
                        routeToMaster: auxSnapshot.routeToMaster,
                        selectedOutputDevice: auxSnapshot.selectedOutputDevice ?? 'no-output',
                        node,
                        outputNode,
                        outputStreamDest,
                        outputAudioContext: null,
                        outputSource: null,
                        // FX
                        reverbNode,
                        reverbEnabled: auxSnapshot.reverbEnabled || false,
                        reverbParams,
                        delayNode,
                        delayEnabled: auxSnapshot.delayEnabled || false,
                        delayParams
                    }

                    auxBuses.value.push(newAux)

                    // Restore output device (must be async, but we don't need to await here)
                    nextTick(() => {
                        const auxIndex = auxBuses.value.findIndex(a => a.id === auxSnapshot.id)
                        if (auxIndex !== -1 && auxSnapshot.selectedOutputDevice !== undefined && auxSnapshot.selectedOutputDevice !== 'no-output') {
                            changeAuxOutputDevice(auxIndex, auxSnapshot.selectedOutputDevice)
                        }
                    })
                })

                // Update nextAuxId counter
                if (scene.auxBuses.length > 0) {
                    const maxId = Math.max(...scene.auxBuses.map((a: AuxSnapshot) => {
                        const match = a.id.match(/aux-(\d+)/)
                        return match ? parseInt(match[1]) : 0
                    }))
                    nextAuxId = maxId + 1
                }
            }

            // Wait for Vue to update props before restoring tracks
            nextTick(() => {
                // Restore each track's state (NOW aux buses exist and props are updated)
                scene.tracks.forEach((trackSnapshot: TrackSnapshot) => {
                    const trackRef = trackRefs.value.get(trackSnapshot.trackNumber)
                    if (trackRef && trackRef.restoreFromSnapshot) {
                        trackRef.restoreFromSnapshot(trackSnapshot)
                    }
                    
                    // Restore track order
                    const track = tracks.value.find(t => t.id === trackSnapshot.trackNumber)
                    if (track && trackSnapshot.order !== undefined) {
                        track.order = trackSnapshot.order
                    }
                })
            })

            // Restore master EQ filters
            if (scene.master.masterEQFilters && scene.master.masterEQFilters.length > 0) {
                if (rightSectionRef.value?.masterEqFiltersData) {
                    rightSectionRef.value.masterEqFiltersData = scene.master.masterEQFilters.map((filter: any) => ({
                        type: filter.type,
                        frequency: filter.frequency,
                        gain: filter.gain,
                        Q: filter.Q,
                        color: filter.color || '#3b82f6'
                    }))
                }
            } else {
                if (rightSectionRef.value?.masterEqFiltersData) {
                    rightSectionRef.value.masterEqFiltersData = []
                }
            }

            // Restore master section state (volumes, FX)
            if (masterSectionRef.value?.restoreSnapshot) {
                masterSectionRef.value.restoreSnapshot(scene.master)
            }

            // Ensure we have enough subgroups to restore the scene
            if (scene.subgroups && scene.subgroups.length > 0) {
                // Create missing subgroups if needed
                const existingIds = new Set(subgroups.value.map(s => s.id))
                const missingSubgroups = scene.subgroups.filter(
                    (snapshot: SubgroupSnapshot) => !existingIds.has(snapshot.id)
                )

                if (missingSubgroups.length > 0) {
                    missingSubgroups.forEach((snapshot: SubgroupSnapshot) => {
                        // Create Tone.js channel for this subgroup
                        const channel = new Tone.Channel({
                            volume: 0,
                            pan: 0,
                            channelCount: 2,
                            channelCountMode: 'explicit',
                            channelInterpretation: 'speakers'
                        })

                        subgroups.value.push({
                            id: snapshot.id,
                            name: snapshot.name,
                            channel,
                            ref: null
                        })
                    })

                    // Wait for components to mount
                    nextTick(() => {
                        setTimeout(() => {
                            restoreSubgroupSnapshots(scene.subgroups!)
                        }, 100)
                    })
                } else {
                    restoreSubgroupSnapshots(scene.subgroups)
                }
            }

            // Restore automation data
            if (scene.automation) {
                automation.importAutomation(scene.automation)
            } else {
                // No automation in scene, clear existing automation
                automation.clearAll()
            }

            // Set as current scene
            setCurrentScene(scene.id)

            // Hide loading overlay after scene is restored and audio files have time to load
            setTimeout(() => {
                isLoadingScene.value = false
            }, 1200) // Extra time for audio files to load silently
        }, 600) // Wait 0.6 seconds at -∞ before restoring values
    }, 200) // Initial delay after closing modal
}

async function handleUpdateScene(sceneId: string) {
    // Get old scene to clean up orphaned files
    const oldScene = scenes.value.find((s: Scene) => s.id === sceneId)

    // Collect current state from tracks
    const trackSnapshots: TrackSnapshot[] = []
    tracks.value.forEach(track => {
        const trackRef = trackRefs.value.get(track.id)
        if (trackRef && trackRef.getSnapshot) {
            trackSnapshots.push(trackRef.getSnapshot())
        }
    })

    // Collect master snapshot from MasterSection
    const masterSectionSnapshot = masterSectionRef.value?.getSnapshot?.() || {
        leftVolume: 0,
        rightVolume: 0,
        headphonesVolume: 0,
        isLinked: true,
        compressorEnabled: false,
        reverbEnabled: false,
        delayEnabled: false,
        limiterEnabled: false
    }

    const masterSnapshot = {
        ...masterSectionSnapshot,
        masterEQFilters: (rightSectionRef.value?.masterEqFiltersData || []).map((filter: any) => ({
            type: filter.type,
            frequency: filter.frequency,
            gain: filter.gain,
            Q: filter.Q,
            color: filter.color
        }))
    }

    // Collect subgroup snapshots
    const subgroupSnapshots: SubgroupSnapshot[] = []
    subgroups.value.forEach(subgroup => {
        if (subgroup.ref && subgroup.ref.getSnapshot) {
            const snapshot = subgroup.ref.getSnapshot()
            subgroupSnapshots.push({
                id: subgroup.id,
                name: subgroup.name,
                ...snapshot
            })
        } else {
            console.warn(`[Scene Update] Subgroup ${subgroup.name} (ID: ${subgroup.id}) has no ref or getSnapshot`)
        }
    })

    // Collect aux snapshots
    const auxSnapshots: AuxSnapshot[] = []
    auxBuses.value.forEach(aux => {
        auxSnapshots.push({
            id: aux.id,
            name: aux.name,
            volume: aux.volume,
            muted: aux.muted,
            soloed: aux.soloed,
            routeToMaster: aux.routeToMaster,
            selectedOutputDevice: aux.selectedOutputDevice,
            // FX
            reverbEnabled: aux.reverbEnabled,
            reverbParams: aux.reverbParams,
            delayEnabled: aux.delayEnabled,
            delayParams: aux.delayParams
        })
    })

    // Update scene
    await updateScene(
        sceneId, 
        trackSnapshots, 
        masterSnapshot, 
        subgroupSnapshots, 
        auxSnapshots,
        automation.exportAutomation() // Add automation data
    )

    // Clean up old audio files that are no longer in the scene
    if (oldScene) {
        const oldFileIds = new Set(
            oldScene.tracks
                .filter((t: TrackSnapshot) => t.fileId)
                .map((t: TrackSnapshot) => t.fileId!)
        )
        const newFileIds = new Set(
            trackSnapshots
                .filter(t => t.fileId)
                .map(t => t.fileId!)
        )

        // Delete files that were in old scene but not in new scene
        const filesToDelete = Array.from(oldFileIds).filter(id => !newFileIds.has(id))

        for (const fileId of filesToDelete) {
            try {
                await deleteAudioFile(fileId)
            } catch (err) {
                console.warn(`Failed to delete orphaned file ${fileId}:`, err)
            }
        }
    }
}

async function handleDeleteScene(sceneId: string) {
    // Find scene to get file IDs before deletion
    const scene = scenes.value.find((s: Scene) => s.id === sceneId)

    if (scene) {
        // Delete all audio files associated with this scene from IndexedDB
        const fileDeletePromises: Promise<void>[] = []

        scene.tracks.forEach((track: TrackSnapshot) => {
            if (track.fileId) {
                fileDeletePromises.push(
                    deleteAudioFile(track.fileId).catch((err: unknown) => {
                        console.warn(`Failed to delete file ${track.fileId}:`, err)
                    })
                )
            }
        })

        // Wait for all file deletions to complete
        await Promise.all(fileDeletePromises)
    }

    // Delete scene from storage (this removes from both scenes array and IndexedDB)
    await deleteScene(sceneId)

    // If deleted scene was current, clear current
    if (currentSceneId.value === sceneId) {
        setCurrentScene(null)
    }
}

async function handleRenameScene(sceneId: string, newName: string) {
    await renameScene(sceneId, newName)
}

async function handleTogglePin(sceneId: string) {
    await togglePinScene(sceneId)
}

function handleClearScene() {
    // Reload the page to clear all mixer state
    window.location.reload()
}

// Computed for pinned scenes
const pinnedScenes = computed(() => {
    return scenes.value.filter(scene => scene.pinned)
})

// Initialize audio
onMounted(async () => {
    document.title = 'Audio Mixer Pro - Multi-Track Mixer'

    // Load automation panel state
    loadAutomationHeight()

    // Load scenes from IndexedDB
    await loadScenesFromStorage()

    // Get Tone.js from inject
    if (ToneRef?.value) {
        Tone = ToneRef.value
    } else {
        // Fallback: wait for it
        await new Promise<void>((resolve) => {
            const checkTone = setInterval(() => {
                if (ToneRef?.value) {
                    Tone = ToneRef.value
                    clearInterval(checkTone)
                    resolve()
                }
            }, 100)
        })
    }

    // Mark Tone as ready immediately after import
    toneReady.value = true

    // Use Gain instead of Channel to preserve stereo
    // Tone.Channel converts stereo to mono!
    masterChannel.value = new Tone.Channel({
        volume: 0,
        pan: 0,
        channelCount: 2,
        channelCountMode: 'explicit',
        channelInterpretation: 'speakers'
    })

    // Add initial subgroup (only if allowed by build limits)
    const limits = getBuildLimits()
    if (limits.maxSubgroups > 0) {
        addSubgroup()
    }

    // Add default aux buses (up to the build limit)
    const maxAuxToAdd = Math.min(6, limits.maxAuxBuses)
    for (let i = 0; i < maxAuxToAdd; i++) {
        addAux()
    }

    // Don't start connection here - wait for component to mount

    // Ensure audio context is running
    if (Tone.context.state !== 'running') {
        await Tone.context.resume()
    }

    // Enumerate audio devices ONCE for all tracks
    // Don't await - let it run in background and don't block app startup
    enumerateAudioInputs().catch((error) => {
        console.warn('[Index] Failed to enumerate audio inputs (likely permission denied):', error)
        // App will still work, but audio input selection might be limited
    })

    // Wait for next tick to ensure all components are ready
    await nextTick()

    // Delay to ensure all components are fully mounted and initialized
    setTimeout(() => {
        isReady.value = true
        isAppReady.value = true // Notify App.vue that everything is ready
    }, 100)

    // Close add track menu when clicking outside
    document.addEventListener('click', (e) => {
        const target = e.target as HTMLElement
        if (!target.closest('.relative')) {
            showAddTrackMenu.value = false
        }
    })
})

// ========================================
// AUTOMATION TRANSPORT CONTROLS
// ========================================

function handlePlay() {
    automation.play()
    startPlaybackLoop()
    
    // Synchronize all audio track playback with automation transport
    trackRefs.value.forEach((trackRef, trackId) => {
        const track = tracks.value.find(t => t.id === trackId)
        // Only sync audio tracks, not signal generators
        if (track?.type === 'audio' && trackRef?.startPlayback) {
            trackRef.startPlayback()
        }
    })
}

function handlePause() {
    automation.pause()
    stopPlaybackLoop()
    
    // Pause all audio track playback
    trackRefs.value.forEach((trackRef, trackId) => {
        const track = tracks.value.find(t => t.id === trackId)
        if (track?.type === 'audio' && trackRef?.pausePlayback) {
            trackRef.pausePlayback()
        }
    })
}

function handleStop() {
    automation.stop()
    stopPlaybackLoop()
    
    // Stop all audio track playback
    trackRefs.value.forEach((trackRef, trackId) => {
        const track = tracks.value.find(t => t.id === trackId)
        if (track?.type === 'audio' && trackRef?.stopPlayback) {
            trackRef.stopPlayback()
        }
    })
}

function toggleRecordMode() {
    if (automation.isRecording.value) {
        // Stop recording
        automation.stopRecording()
        
        // Set all lanes to READ mode
        automation.automationLanes.value.forEach(lane => {
            lane.mode = 'read'
            lane.enabled = true
        })
    } else {
        // Start recording - create/enable automation lanes only for armed tracks
        const tracksToRecord = tracks.value.filter(track => 
            track.type === 'audio' && armedTracks.value.has(track.id)
        )
        
        if (tracksToRecord.length === 0) {
            // No tracks armed - show alert
            alert('⚠️ Nessuna traccia armata!\n\nClicca il pulsante "ARM" su una o più tracce per registrare l\'automazione.')
            return
        }
        
        tracksToRecord.forEach(track => {
            // Create volume lane in WRITE mode
            const volumeLane = automation.getOrCreateLane(track.id, 'volume')
            volumeLane.mode = 'write'
            volumeLane.enabled = true
        })
        
        automation.isRecording.value = true
        
        // Auto-start playback if not already playing
        if (!automation.transport.value.isPlaying) {
            handlePlay()
        }
    }
}

function startPlaybackLoop() {
    if (playbackLoopId.value) return // Already running
    
    const updateInterval = 1000 / 60 // 60 Hz update rate
    let lastTime = Date.now()
    
    const loop = () => {
        if (automation.transport.value.isPlaying) {
            const now = Date.now()
            const delta = (now - lastTime) / 1000 // seconds
            lastTime = now
            
            // Update transport time
            automation.transport.value.currentTime += delta
            
            // Check for loop
            if (automation.transport.value.currentTime >= automation.transport.value.duration) {
                if (loopEnabled.value) {
                    automation.transport.value.currentTime = 0
                } else {
                    handleStop()
                    return
                }
            }
            
            // Apply automation to tracks
            applyAutomationToTracks()
            
            playbackLoopId.value = requestAnimationFrame(loop)
        }
    }
    
    playbackLoopId.value = requestAnimationFrame(loop)
}

function stopPlaybackLoop() {
    if (playbackLoopId.value) {
        cancelAnimationFrame(playbackLoopId.value)
        playbackLoopId.value = null
    }
}

function applyAutomationToTracks() {
    const currentTime = automation.transport.value.currentTime
    
    // Apply volume and pan automation to all audio tracks
    tracks.value.forEach(track => {
        if (track.type !== 'audio') return // Skip signal generator tracks
        
        const volumeValue = automation.getValueAtTime(track.id, 'volume', currentTime)
        if (volumeValue !== null && trackRefs.value.has(track.id)) {
            const trackRef = trackRefs.value.get(track.id)
            // Set volume without triggering automation recording
            if (trackRef?.setVolume) {
                trackRef.setVolume(volumeValue, true) // true = skip automation recording
            }
        }
        
        // Apply pan automation
        const panValue = automation.getValueAtTime(track.id, 'pan', currentTime)
        if (panValue !== null && trackRefs.value.has(track.id)) {
            const trackRef = trackRefs.value.get(track.id)
            if (trackRef?.setPan) {
                trackRef.setPan(panValue, true)
            }
        }
    })
}

// Automation panel resize/collapse functions
function toggleAutomationCollapse() {
    if (automationCollapsed.value) {
        // Expand
        automationHeight.value = automationSavedHeight.value || 300
        automationCollapsed.value = false
    } else {
        // Collapse
        automationSavedHeight.value = automationHeight.value
        automationHeight.value = 12 // Collapsed height (just the handle)
        automationCollapsed.value = true
    }
    saveAutomationHeight()
}

function startAutomationResize(event: MouseEvent) {
    if (automationCollapsed.value) return // Don't allow resize when collapsed
    
    automationResizing = true
    automationStartY = event.clientY
    automationStartHeight = automationHeight.value
    
    document.addEventListener('mousemove', onAutomationResize)
    document.addEventListener('mouseup', stopAutomationResize)
    
    // Prevent text selection during resize
    event.preventDefault()
    document.body.style.cursor = 'ns-resize'
    document.body.style.userSelect = 'none'
}

function onAutomationResize(event: MouseEvent) {
    if (!automationResizing) return
    
    // Calculate new height (drag up = increase height, drag down = decrease height)
    const deltaY = automationStartY - event.clientY
    const maxHeight = window.innerHeight * 0.7 // Max: 70% of window height
    const newHeight = Math.max(150, Math.min(maxHeight, automationStartHeight + deltaY)) // Min 150px
    
    // Store pending height
    automationPendingHeight = newHeight
    
    // Use RAF to throttle updates to once per frame
    if (automationResizeRafId === null) {
        automationResizeRafId = requestAnimationFrame(() => {
            if (automationPendingHeight !== null) {
                automationHeight.value = automationPendingHeight
                automationPendingHeight = null
            }
            automationResizeRafId = null
        })
    }
}

function stopAutomationResize() {
    if (automationResizing) {
        automationResizing = false
        document.removeEventListener('mousemove', onAutomationResize)
        document.removeEventListener('mouseup', stopAutomationResize)
        document.body.style.cursor = ''
        document.body.style.userSelect = ''
        
        // Cancel any pending RAF and apply final height
        if (automationResizeRafId !== null) {
            cancelAnimationFrame(automationResizeRafId)
            automationResizeRafId = null
        }
        if (automationPendingHeight !== null) {
            automationHeight.value = automationPendingHeight
            automationPendingHeight = null
        }
        
        // Save to localStorage
        saveAutomationHeight()
    }
}

function saveAutomationHeight() {
    try {
        localStorage.setItem('automationPanelHeight', automationHeight.value.toString())
        localStorage.setItem('automationPanelCollapsed', automationCollapsed.value.toString())
        if (!automationCollapsed.value) {
            localStorage.setItem('automationPanelSavedHeight', automationSavedHeight.value.toString())
        }
    } catch (err) {
        console.warn('Failed to save automation panel height:', err)
    }
}

function loadAutomationHeight() {
    try {
        const saved = localStorage.getItem('automationPanelHeight')
        const collapsed = localStorage.getItem('automationPanelCollapsed')
        const savedHeightStr = localStorage.getItem('automationPanelSavedHeight')
        
        const maxHeight = window.innerHeight * 0.7
        
        if (saved) {
            const height = Math.max(150, Math.min(maxHeight, parseInt(saved)))
            automationHeight.value = height
        }
        
        if (collapsed === 'true') {
            automationCollapsed.value = true
            automationHeight.value = 12
        } else if (collapsed === 'false') {
            automationCollapsed.value = false
        }
        
        if (savedHeightStr) {
            automationSavedHeight.value = Math.max(150, Math.min(maxHeight, parseInt(savedHeightStr)))
        }
    } catch (err) {
        console.warn('Failed to load automation panel height:', err)
    }
}

// Cleanup on unmount
onUnmounted(() => {
    stopPlaybackLoop()
})
</script>

<style scoped>
.mixer-app {
    height: 100vh;
    overflow: hidden;
}

.mixer-fade-in {
    animation: fadeIn 0.2s ease-in;
}

@keyframes fadeIn {
    from {
        opacity: 0;
    }

    to {
        opacity: 1;
    }
}

/* Custom scrollbar */
::-webkit-scrollbar {
    width: 8px;
    height: 8px;
}

::-webkit-scrollbar-track {
    background: #1f2937;
}

::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
}

/* Tracks section horizontal scrollbar - thin 4px */
.tracks-scroll::-webkit-scrollbar {
    height: 4px;
}

.tracks-scroll::-webkit-scrollbar-track {
    background: rgba(31, 41, 55, 0.3);
    border-radius: 2px;
}

.tracks-scroll::-webkit-scrollbar-thumb {
    background: rgba(59, 130, 246, 0.5);
    border-radius: 2px;
}

.tracks-scroll::-webkit-scrollbar-thumb:hover {
    background: rgba(96, 165, 250, 0.8);
}

.tracks-scroll-wrap {
    position: relative;
}

/* Track drag and drop styles */
.track-wrapper {
    transition: opacity 0.2s ease, transform 0.2s ease;
}

.track-wrapper.dragging {
    opacity: 0.5;
}

.track-wrapper.drag-over {
    transform: translateX(4px);
}

.track-wrapper.drag-over::before {
    content: '';
    position: absolute;
    left: -2px;
    top: 0;
    bottom: 0;
    width: 4px;
    background: linear-gradient(180deg, #3b82f6 0%, #8b5cf6 100%);
    border-radius: 2px;
    z-index: 10;
    animation: pulse-glow 1s ease-in-out infinite;
}

@keyframes pulse-glow {
    0%, 100% {
        opacity: 1;
        box-shadow: 0 0 8px rgba(59, 130, 246, 0.6);
    }
    50% {
        opacity: 0.7;
        box-shadow: 0 0 12px rgba(59, 130, 246, 0.8);
    }
}

.tracks-scroll-wrap::after {
    content: '';
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 36px;
    pointer-events: none;
    z-index: 5;
    background: linear-gradient(to left, rgba(0, 0, 0, 0.95), rgba(0, 0, 0, 0));
}

/* Draggable components */
[draggable="true"] {
    transition: all 0.3s ease;
    border-radius: 0.5rem;
}

[draggable="true"]:hover {
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

[draggable="true"]:active {
    opacity: 0.7;
    transform: scale(0.98);
}
</style>
