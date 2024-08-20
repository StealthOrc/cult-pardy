



export function getHttpUrl(url: string): string {
    const isHttps = location.protocol === 'https:';
    return  `${isHttps ? 'https' : 'http'}://${location.host}${url}`;
}


export function get_global_time(delta: number) {
    //UTC time

    
    const d = new Date();
    return d.getTime() + delta;
}


export function timeout(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}