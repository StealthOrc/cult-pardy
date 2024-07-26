import type { PageLoad } from './$types';

export const load: PageLoad = ({route, params}) => {
    console.log("PageLoad: ", route.id, params);
    return {};
};