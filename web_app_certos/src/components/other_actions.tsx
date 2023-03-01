import { Button, Box } from '@mui/material';
import CardTitle from './card_title';

const OtherActions = () => {

    return(<>
        <CardTitle text="Other actions" />
        <Box sx={{my: 2, mx: 3}}>
            <Button
                href="#/Template"
                id="templates"
            >
                See all templates
            </Button>
        </Box>
    </>)
}

export default OtherActions;