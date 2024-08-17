export const prerender = false;
import type { PageLoad } from './$types';

export const load: PageLoad = ({params}) => {
    const { lobbyid } = params;
    console.log(lobbyid);
    return { lobbyid };
}