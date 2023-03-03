import { Button, Box } from '@mui/material';
import CardTitle from './card_title';
import { useTranslate } from 'react-admin';

const OtherActions = () => {
    const translate = useTranslate();
    return(<>
        <CardTitle text="certos.dashboard.other_actions.title" />
        <Box sx={{my: 2, mx: 3}}>
            <Button
                href="#/Template"
                id="templates"
            >
                {translate("certos.dashboard.other_actions.content")}
            </Button>
        </Box>
    </>)
}

export default OtherActions;