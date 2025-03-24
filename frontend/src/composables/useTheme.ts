import { darkTheme, lightTheme} from 'naive-ui';
import { ref } from 'vue';
const theme = ref(darkTheme);
export const useTheme = () =>
{
    const dark_theme = () =>
    {
        theme.value = darkTheme;
    }
    const light_theme = () =>
    {
        theme.value = lightTheme;
    }
    return {dark_theme, light_theme, theme}
}