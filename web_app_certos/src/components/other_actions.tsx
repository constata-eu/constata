import { Button, Box } from '@mui/material';
import CardTitle from './card_title';
import { useTranslate } from 'react-admin';

const OtherActions = () => {
  const translate = useTranslate();
  return(<>
    <CardTitle text="certos.dashboard.other_actions.title" />
    <Box sx={{my: 2, mx: 3}}>
      <Button
        sx={{display: "block"}}
        href="#/Template"
        id="templates"
      >
        {translate("certos.dashboard.other_actions.templates")}
      </Button>
      <Button 
        sx={{display: "block"}}
        href="#/graphiql"
        id="graphiql"
      >
        {translate("certos.dashboard.other_actions.graphiql")}
      </Button>
      <Button 
        sx={{display: "block"}}
        href="#/vc_prompt_dashboard"
        id="vc_prompt_dashboard"
      >
        Puntos de verificación
      </Button>
    </Box>
  </>)
}

export default OtherActions;
