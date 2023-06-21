import {Card, CardContent, Skeleton} from '@mui/material';
import CardTitle from './card_title';
import _ from "lodash";

const ConstataSkeleton = ({title, ...rest}) => {
  return (<Card sx={{ mb: 5 }}>
    <CardTitle text={title}/>
    <CardContent>
      {_.range(rest.lines || 2).map(i => <Skeleton key={i} />)}
      <Skeleton width="60%"/>

      <Skeleton variant="rectangular" height="2em" sx={{mt:2}} />
    </CardContent>
  </Card>);
}

export default ConstataSkeleton;
