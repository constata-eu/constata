import {AppBar, Toolbar, IconButton, Box, Button, Container, styled, Backdrop, Skeleton, useMediaQuery } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';
import { useNavigate } from 'react-router-dom';
import { useTranslate, useSafeSetState, useGetOne} from 'ra-core';
import { useLogout } from "react-admin";
import DashboardIcon from "@mui/icons-material/Dashboard";
import MenuIcon from '@mui/icons-material/Menu';
import CloseIcon from '@mui/icons-material/Close';
import logo from '../assets/logo_denim.png';

const ResponsiveAppBar = ({loggedIn, accountState}) => {
  const [menuOpen, setMenuOpen] = useSafeSetState(false);
  const logout = useLogout();
  const navigate = useNavigate();
  const translate = useTranslate();
  const isSmall = useMediaQuery((theme: any) => theme.breakpoints.down('sm'));

  const MobileMenu = () => <Box sx={{ flexGrow: 1, display: { xs: 'flex', md: 'none' }, justifyContent: "end" }} id="mobile-menu">
    <IconButton
      size="large"
      aria-controls="menu-appbar"
      onClick={() => setMenuOpen(true) }
      color="inherit"
    >
      <MenuIcon />
    </IconButton>
    <Backdrop
      sx={{ color: '#fff', backgroundColor:"rgba(0, 0, 0, 0.9)", zIndex: (theme) => theme.zIndex.drawer + 1 }}
      open={menuOpen}
      onClick={() => setMenuOpen(false) }
    >
      <Box display="flex" flexDirection="column">
        <IconButton sx={{ "svg": { fontSize: "80px !important" }, mb: 2 }} color="inverted" onClick={ () => setMenuOpen(false) } >
          <CloseIcon />
        </IconButton>

        { !accountState?.useVerifier &&
          <Button size="large" sx={{ "svg": { fontSize: "1em !important" }, fontSize: 40, mb: 2, textTransform: "uppercase"}} color="inverted" onClick={ () => navigate("/") } id="dashboard-mobile-menu-item"
            startIcon={<DashboardIcon/>} 
          >
            { translate("certos.menu.dashboard") }
          </Button>
        }

        { !accountState?.useVerifier &&
          <Button size="large" sx={{ fontSize: 40, mb: 2, textTransform: "uppercase"}} color="inverted" onClick={ () => navigate("/Issuance") } id="issuances-mobile-menu-item">
            { translate("certos.menu.issuances") }
          </Button>
        }
        <Button size="large"sx={{ fontSize: 40, mb: 2, textTransform: "uppercase"}} color="inverted" href="mailto:hola@constata.eu" target="_blank" id="help-mobile-menu-item">
          { translate("certos.menu.help") }
        </Button>
        <Button size="large" sx={{ fontSize: 40, mb: 2, textTransform: "uppercase" }} color="inverted" onClick={() => logout() } id="logout-mobile-menu-item">
          { translate("certos.menu.logout") }
        </Button>
      </Box>
    </Backdrop>
  </Box>

  const ComputerMenu = () => <Box sx={{ flexGrow: 1, display: { xs: 'none', md: 'flex' }, justifyContent:"end" }} id="desktop-menu">
    { !accountState?.useVerifier &&
      <Button sx={{ ml: 1, textTransform: "uppercase" }} variant="contained" color="highlight" onClick={ () => navigate("/") } startIcon={<DashboardIcon/>} id="dashboard-menu-item">
        { translate("certos.menu.dashboard") }
      </Button>
    }
    { !accountState?.useVerifier &&
      <Button sx={{ ml: 1, textTransform: "uppercase" }} color="highlight" onClick={ () => navigate("/Issuance") } id="issuances-menu-item">
        { translate("certos.menu.issuances") }
      </Button>
    }
    <Button sx={{ ml: 1, textTransform: "uppercase" }} color="highlight" href="mailto:hola@constata.eu" target="_blank" id="help-menu-item">
      { translate("certos.menu.help") }
    </Button>
    <Button sx={{ ml: 1, textTransform: "uppercase" }} variant="outlined" color="highlight" onClick={() => logout() } id="logout-menu-item">
      { translate("certos.menu.logout") }
    </Button>
  </Box>

  return (
    <AppBar position="static" color="inverted" sx={{ py: isSmall ? "14px" : "28px"}}>
      <Container maxWidth="md" style={{ padding: 0}}>
        <Toolbar sx={{ minHeight: "0 !important" }}>
          <Box sx={{ display: "flex"}} >
            { 
              (accountState?.logoUrl) ?
                <img src={accountState.logoUrl} style={{ maxHeight: isSmall ? "30px" : "65px", width: "auto" }}/>
            
              :
              <a href="https://constata.eu" style={{lineHeight: 0}} target="_blank" rel="noreferrer">
                <img src={logo} alt={translate("certos.menu.logo")} style={{ height: isSmall ? "20px" : "30px", width: "auto" }}/>
              </a>
            }
          </Box>
          {loggedIn && <>
              <MobileMenu />
              <ComputerMenu />
            </>
          }
        </Toolbar>
      </Container>
    </AppBar>
  );
}

const Root = styled("div")(({ theme }) => ({
  display: "flex",
  flexDirection: "column",
  zIndex: 1,
  minHeight: "100vh",
  backgroundColor: theme.palette.background.default,
  position: "relative",
}));

const AppFrame = styled("div")(() => ({
  display: "flex",
  flexDirection: "column",
  overflowX: "auto",
  alignItems: "center",
  marginBottom: "3em",
}));

const Content = styled("div")(({ theme }) => ({
  width: "100%",
  display: "flex",
  flexDirection: "column",
  overflowX: "auto",
  paddingTop: "2em",
}));

export const BareLayout = ({children}) => {
  const translate = useTranslate();
  return (<Box sx={{
      minHeight: "100vh",
      display: "flex",
      flexDirection: "column",
    }}>
      <CssBaseline/>
      <Container maxWidth="sm">
        { children }
        <Box textAlign="center" mt={8} mb={4}>
          <a href="https://constata.eu" style={{lineHeight: 0}} target="_blank" rel="noreferrer">
            <img src={logo}  alt={translate("certos.menu.logo")} style={{width: "200px" }} />
          </a>
        </Box>
      </Container>
    </Box>
  )
}

export const ToolbarLayout = ({children, loggedIn, accountState}) => {
  return (
    <Root>
      <CssBaseline/>
      <AppFrame>
        <ResponsiveAppBar loggedIn={loggedIn} accountState={accountState} />
        <Content>
          {children}
        </Content>
      </AppFrame>
    </Root>
  )
}

export const NoLoggedInLayout = ({ children }) => {
  return <ToolbarLayout loggedIn={false} children={children} />;
};

export const ConstataLayout = ({ children }) => {
  const {isLoading, data: accountState} = useGetOne( 'AccountState', { id: 1 });

  if (isLoading) return <Container>
    <Skeleton/>
    <Skeleton/>
    <Skeleton/>
  </Container>;
  
  return <ToolbarLayout loggedIn={true} accountState={accountState} children={children} />;
};