import { styled } from '@mui/material/styles';
import Tooltip, { tooltipClasses } from '@mui/material/Tooltip';

const BootstrapTooltip = styled(({ className, ...props }: any) => (
    <Tooltip {...props} arrow classes={{ popper: className }} />
  ))(() => ({
    [`& .${tooltipClasses.arrow}`]: {
      color: '#1163c2',
    },
    [`& .${tooltipClasses.tooltip}`]: {
      backgroundColor: '#1163c2',
    },
  }));

export default BootstrapTooltip;