export const formatDuration = (val) => {
    if (val == null || val === '') return '0:00';
    const strVal = String(val);

    if (strVal.includes(':')) return strVal;

    const s = parseInt(strVal, 10);
    if (Number.isNaN(s) || s < 0) return '0:00';

    const mins = Math.floor(s / 60);
    const secs = String(s % 60).padStart(2, '0');

    return `${mins}:${secs}`;
};