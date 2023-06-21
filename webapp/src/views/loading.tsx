import { CircularProgress, Box } from '@mui/material';

const Loading = () => {
  return(
    <Box display="flex" alignItems="center" justifyContent="center" height="100vh" >
      <CircularProgress />
    </Box>
  )
}

export default Loading;