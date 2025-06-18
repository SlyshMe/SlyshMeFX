<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getVersion } from "@tauri-apps/api/app";
    import { type Monitor } from "@tauri-apps/api/window";
    
    import * as Command from "$lib/components/ui/command";
    import { Slider } from "$lib/components/ui/slider";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import { Button } from "$lib/components/ui/button";
    import * as Select from "$lib/components/ui/select";     
    import ColourPicker from "svelte-awesome-color-picker";

    import type { Configs, EqualiserSettings, GainType, VisualiserSettings } from "$lib/types";

    let src: string = $state(``);
    let visualiserSettings: VisualiserSettings = $state({
        barsColour: [0, 0, 0, 170],
        visualiserType: `Linear1`,
        useDesktopBackground: true,
        resolution: 128,
        screen: undefined,
    });
    let equaliserSettings: EqualiserSettings = $state([
        {
            preamp: 0,
            channelLeft: true,
            bassGain: 0,
            lowGain: 0,
            midGain: 0,
            highGain: 0,
            trebleGain: 0,
        },
        {
            preamp: 0,
            channelLeft: false,
            bassGain: 0,
            lowGain: 0,
            midGain: 0,
            highGain: 0,
            trebleGain: 0,
        }
    ]);
    let rgb = $state({ r: 0, g: 0, b: 0, a: 170 / 255 });


    invoke(`getConfigs`).then((e) => {
        const configs = e as Configs;
        equaliserSettings = configs[0];
        visualiserSettings = configs[1];
    });
    invoke(`getWallpaper`).then((v) => {
        const data = new Uint8Array(v as Array<number>);
        const blob = new Blob([data], { type: "image/png" });
        const url = URL.createObjectURL(blob);

        src = url;
    }).catch(console.log);

    $effect(() => {
        invoke(`setEqualiserSettings`, {
            newSettings: JSON.stringify(equaliserSettings)
        }).then(console.log).catch(console.log)
    });
    $effect(() => {
        invoke(`setVisualiserSettings`, {
            newSettings: JSON.stringify(visualiserSettings)
        }).then(console.log).catch(console.log);
    });
    $effect(() => {
        rgb = {
            r: visualiserSettings.barsColour[0],
            g: visualiserSettings.barsColour[1],
            b: visualiserSettings.barsColour[2],
            a: visualiserSettings.barsColour[3] / 255,
        };
    });

    let sliders: Array<{ value: GainType, label: string }> = [
        {
            value: `preamp`,
            label: `Volume`,
        },
        {
            value: `bassGain`,
            label: `Bass`,
        },
        {
            value: `lowGain`,
            label: `Low`,
        },
        {
            value: `midGain`,
            label: `Mid`,
        },
        {
            value: `highGain`,
            label: `High`,
        },
        {
            value: `trebleGain`,
            label: `Treble`,
        }
    ];

    let hovers = {
        wrapper: true,
        select: false,
        select2: false,
    };
    const toggleHovers = (hoverType: `wrapper` | `select` | `select2`, value: boolean) => {
        hovers[hoverType] = value;

        if (!hovers.wrapper && !hovers.select && !hovers.select2) setTimeout(() => !hovers.wrapper && !hovers.select ? invoke(`hideSettingsUi`) : null, 250);
    };
</script>

<div 
    class="wrapper" 
    onmouseleave={() => toggleHovers(`wrapper`, false)} 
    onmouseenter={() => toggleHovers(`wrapper`, true)} 
    role="document"
>
    <img src="{src}" style="display: {visualiserSettings.useDesktopBackground ? `block` : `none`};" alt="">

    <div class="content">
        <div class="glass">
            <Command.Root class="bg-transparent text-white overflow-visible">
                <Command.List class="max-h-[1000px] overflow-visible">
                    <Command.Group heading="Visualiser" class="overflow-visible">
                        <Command.Item class="dark flex justify-between">
                            Colour:
                            <ColourPicker
                                {rgb}
                                label=""
                                --picker-z-index="1000" 
                                --picker-height="100px" 
                                --picker-width="100px" 
                                onInput={(colour) => visualiserSettings.barsColour = [colour.rgb?.r ?? 0, colour.rgb?.g ?? 0, colour.rgb?.b ?? 0, colour.rgb?.a ? Math.round(colour.rgb.a * 255) : 170]} 
                            />
                        </Command.Item>
                        <Command.Item class="flex justify-between pr-4">
                            Use wallpaper:
                            <Checkbox checked={visualiserSettings.useDesktopBackground} onCheckedChange={(checked) => visualiserSettings.useDesktopBackground = checked} />
                        </Command.Item>
                        <Command.Item>
                            Resolution:
                            <Slider type="single" value={visualiserSettings.resolution} max={256} min={32} step={2} onValueCommit={(value: number) => visualiserSettings.resolution = value} />
                        </Command.Item>
                        <Command.Item class="flex justify-between">
                            Type:
                            <Select.Root 
                                type="single"
                                bind:value={visualiserSettings.visualiserType}
                                onOpenChange={(open) => toggleHovers(`select`, open)}
                            >
                                <Select.Trigger>
                                    {visualiserSettings.visualiserType}
                                </Select.Trigger>
                                <Select.Content class="max-w-fit">
                                    <Select.Item value="Linear1">Linear 1</Select.Item>
                                    <Select.Item value="Linear2">Linear 2</Select.Item>
                                    <Select.Item value="Log">Logarithmic</Select.Item>
                                </Select.Content>
                            </Select.Root>
                        </Command.Item>
                        <Command.Item class="flex justify-between">
                            Screen:
                            <Select.Root 
                                type="single"
                                bind:value={visualiserSettings.screen}
                                onOpenChange={(open) => toggleHovers(`select2`, open)}
                            >
                                <Select.Trigger>
                                    {visualiserSettings.screen ?? `Automatic`}
                                </Select.Trigger>
                                <Select.Content>
                                    {#await invoke(`getMonitors`) then m}
                                        {@const monitors = JSON.parse(m as string) as Monitor[]}
                                        {#each monitors as monitor}
                                            <Select.Item value={monitor.name ?? "\\\\.\\DISPLAY1"}>
                                                {monitor.name?.replace(`\\\\.\\`, ``)}
                                            </Select.Item>
                                        {/each}
                                    {:catch}
                                        <Select.Item disabled={true} value={""}>
                                            No monitors found.
                                        </Select.Item>
                                    {/await}
                                </Select.Content>
                            </Select.Root>
                        </Command.Item>
                    </Command.Group>
                    <Command.Group heading="Equaliser" class="z-0">
                        <Command.Item class="cursor-pointer" onSelect={() => invoke("setupEqualiser")}>
                            Install equaliser
                        </Command.Item>
                        {#each sliders as slider}
                            <Command.Item class="flex justify-between">
                                {slider.label}:
                                <Slider class="max-w-[90px]" type="single" value={equaliserSettings[0][slider.value]} max={20} min={-20} step={1} onValueCommit={(value: number) => {
                                    equaliserSettings[0][slider.value] = value;
                                    equaliserSettings[1][slider.value] = value;
                                }} />
                            </Command.Item>
                        {/each}
                    </Command.Group>
                    <!-- <Command.Group heading="Configurations">
                        Going to implement this later...
                    </Command.Group> -->
                    {#await getVersion() then version}
                        <Command.Group heading="SlyshMeFX v{version}">
                            <Command.Item class="flex justify-between">
                                <Button variant="secondary" onclick={() => invoke(`close`, { restart: true })}>Restart</Button>
                                <Button variant="secondary" onclick={() => invoke(`close`, { restart: false })}>Close</Button>
                            </Command.Item>
                        </Command.Group>
                    {/await}
                </Command.List>
            </Command.Root>
        </div>
    </div>
</div>

<style>
    .content {
        z-index: 2;
        position: absolute;
        top: 0;
        left: 0;

        width: 100vw;
        height: 100vh;

        display: flex;
        align-items: center;
        justify-content: center;

        padding: 5px;

        background-color: #000d;
    }

    .wrapper {
        position: fixed;
        top: 0;
        left: 0;

        width: 100vw;
        height: 100vh;

        border-radius: 10px;
        background-color: #0003;

        overflow: hidden;

        display: flex;
        align-items: center;
        justify-content: center;
    }

    img {
        position: absolute;
        top: 0;
        left: 0;

        width: 100vw;
        height: 100vh;

        overflow: hidden;
        object-fit: cover;

        filter: blur(7.5px);
    }

    .glass {
        background: rgba(0, 0, 0, 0.2);
        border-radius: 7px;
        box-shadow: 0 4px 30px rgba(30, 0, 255, 0.1);
        color: #fff;

        width: calc(100%);
        height: calc(100%);
    }

    * {
		--cp-bg-color: #222;
		--cp-text-color: white;
		--cp-input-color: #333;
		--cp-button-hover-color: #777;
	}

    :global(span.color-picker div.text-input) {
        display: none;
    }
</style>
