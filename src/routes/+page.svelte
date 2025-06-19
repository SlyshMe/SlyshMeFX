<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen, type Event } from "@tauri-apps/api/event";

    import wallpaper from "tauri-plugin-wallpaper";

    import type { BackgroundElements, CanvasPosition, Configs, VisualiserSettings } from "$lib/types";

    let elements: BackgroundElements = {
        canvas: null,
        image: null,
    };
    let image = $state(``);
    let ctx: CanvasRenderingContext2D = $state(elements.canvas?.getContext(`2d`)!);
    let colour = $state(`#000a`);
    let settings: VisualiserSettings = $state({
        barsColour: [0, 0, 0, 170],
        visualiserType: `Linear1`,
        useDesktopBackground: true,
        resolution: 128,
        screen: undefined,
    });
    let canvasPos: CanvasPosition = $state([
        { x: 0, y: 0 },
        { width: window.innerWidth, height: window.innerHeight },
    ]);
    listen(`visualiserUpdate`, (e: Event<String>) => settings = JSON.parse(e.payload.toString()));
    
    invoke(`getConfigs`).then((e) => {
        const configs = e as Configs;
        settings = configs[1];
    });
    
    $effect(() => {
        colour = `#${settings.barsColour[0].toString(16).padStart(2, `0`)}${settings.barsColour[1].toString(16).padStart(2, `0`)}${settings.barsColour[2].toString(16).padStart(2, `0`)}${settings.barsColour[3].toString(16).padStart(2, `0`)}`;
    });
    
    // Attach as 'moving wallpaper'
    wallpaper.attach();
    listen(`startScreenChange`, (e: Event<string | null>) => {
        if (!e.payload) return;

        invoke(`setMonitor`, { monitorName: e.payload }).catch(console.log);
    });
    listen(`setCanvasPosition`, (e: Event<Array<string>>) => {
        if (!e.payload) return;

        canvasPos = [JSON.parse(e.payload[0]), JSON.parse(e.payload[1])];
    });
    
    const prepCanvas = (canvas: HTMLCanvasElement) => {
        // Set resolution based on size
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
    };
    
    const drawImage = (ctx: CanvasRenderingContext2D, image: HTMLImageElement) => {
        const canvas = ctx.canvas;
        const imgWidth = image.naturalWidth;
        const imgHeight = image.naturalHeight;

        const canvasRatio = canvas.width / canvas.height;
        const imageRatio = imgWidth / imgHeight;

        let sx, sy, sWidth, sHeight;

        if (imageRatio > canvasRatio) {
            sHeight = imgHeight;
            sWidth = imgHeight * canvasRatio;
            sx = (imgWidth - sWidth) / 2;
            sy = 0;
        } else {
            sWidth = imgWidth;
            sHeight = imgWidth / canvasRatio;
            sx = 0;
            sy = (imgHeight - sHeight) / 2;
        }

        ctx.drawImage(image, sx, sy, sWidth, sHeight, 0, 0, canvas.width, canvas.height);
    };
    
    invoke("getWallpaper").then(async (v) => {
        const data = new Uint8Array(v as Array<number>);
        const blob = new Blob([data], { type: "image/png" });
        const url = URL.createObjectURL(blob);

        const image = elements.image!;
        const canvas = elements.canvas!;
        ctx = elements.canvas!.getContext(`2d`)!;
        ctx.fillStyle = colour;
        
        await new Promise<void>((resolve, reject) => {
            image.onload = () => resolve();
            image.onerror = (e) => console.log(`Failed to load image`, e) ?? reject(e);

            image.src = url;
        });

        let lastFrame: any = null;
        let highest: number = 1;
        listen(`spectrum`, (e: Event<Array<string>>) => {
            if (highest > 1) highest -=0.01;
            
            const data = e.payload.map((f: string) => JSON.parse(f));
            if (data.length === 0) return;
            if (!lastFrame) lastFrame = data;

            ctx.clearRect(0, 0, canvas.width, canvas.height);
            // drawImage(ctx, image);
            
            ctx.beginPath();
            ctx.moveTo(0, canvas.height);
            
            let x = 0;
            data.forEach((item, index) => {
                if (item.volume > highest) highest = item.volume;
                let barHeight = (.1 + (item.volume / highest)) * canvas.height * .8;

                ctx.lineTo(x, canvas.height - barHeight);
                ctx.lineTo(x + (canvas.width / data.length * 1.1), canvas.height - barHeight);

                x += canvas.width / data.length * 1.1;
            });

            if (data.length == lastFrame.length) lastFrame = data;

            ctx.lineTo(canvas.width, canvas.height);
            ctx.closePath();

            ctx.fillStyle = colour;
            ctx.fill();
        });

        prepCanvas(canvas);

        // need to make sure the previous capture stops when this is called
        invoke(`startCapture`).then(() => console.log(`started capture`)).catch(() => console.log(`failed to start capture`))
    }).catch((e) => console.log(e) ?? console.log(e?.stack));

</script>

<div>
    <img bind:this={elements.image} style="display: {settings.useDesktopBackground ? `block` : `none`};" alt="">
    <canvas style="top: {canvasPos[0].y}px; left: {canvasPos[0].x}px; width: {canvasPos[1].width}px; height: {canvasPos[1].height}px;" bind:this={elements.canvas} id="visualiser"></canvas>
</div>

<style>
    div {
        position: fixed;
        top: 0;
        left: 0;

        width: 100vw;
        height: 100vh;

        overflow: hidden;

        background-color: #1110;
    }

    img {
        position: absolute;
        top: 0;
        left: 0;

        z-index: 1;

        width: 100vw;
        height: 100vh;

        overflow: hidden;
        object-fit: cover;
    }

    canvas {
        position: absolute;

        z-index: 2;
        overflow: hidden;

        background-color: #0000;
    }
</style>
