export interface BackgroundElements {
    canvas: HTMLCanvasElement | null;
    image: HTMLImageElement | null;
};

export type GainType = `preamp` | `bassGain` | `lowGain` | `midGain` | `highGain` | `trebleGain`;
export interface EqualiserChannelSettings {
    preamp: number;
    channelLeft: boolean;
    bassGain: number;
    lowGain: number;
    midGain: number;
    highGain: number;
    trebleGain: number;
};

export type BarsColour = [number, number, number, number];
export type VisualiserType = `Linear1` | `Linear2` | `Log`;
export interface VisualiserSettings {
    barsColour: BarsColour;
    visualiserType: VisualiserType;
    useDesktopBackground: boolean;
    resolution: number;
    screen: string | undefined;
}

export type EqualiserSettings = [EqualiserChannelSettings, EqualiserChannelSettings];
export type Configs = [EqualiserSettings, VisualiserSettings];
