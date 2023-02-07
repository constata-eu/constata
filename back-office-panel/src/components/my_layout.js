import * as React from "react";
import {forwardRef} from "react";
import { Link } from 'react-router-dom';
import { Layout, AppBar, UserMenu, useGetIdentity, Logout, useUserMenu,
         useTranslate } from "react-admin";
import {Avatar, MenuItem, ListItemIcon, ListItemText, Box} from "@mui/material";
import VpnKeyIcon from '@mui/icons-material/VpnKey';


const ChangePassword = forwardRef((props, ref) => {
  const { onClose } = useUserMenu();
  const translate = useTranslate();
  return (
    <MenuItem
        {...props}
        component={Link}
        to="/ChangePassword"
        onClick={onClose}
      >
          <ListItemIcon>
            <VpnKeyIcon/>
          </ListItemIcon>
          <ListItemText>
            {translate("resources.actions.changePassword")}
          </ListItemText>   
    </MenuItem>
  );
});

const MyCustomIcon = () => {
  const { identity } = useGetIdentity();
  if (!identity?.username) return <Avatar />;
  return (
    <>
      <Box component="div" sx={{fontSize: '16px', fontWeight: '600', marginRight: '10px' }}>
        {identity.username}
      </Box>
      <Avatar />
    </>
  );
};

const MyUserMenu = props => (
  <UserMenu {...props}>
    <ChangePassword />
    <Logout />
  </UserMenu>
);

const MyAppBar = props => (
  <AppBar {...props} userMenu={<MyUserMenu icon={<MyCustomIcon />} />} />
);

const MyLayout = props => <Layout {...props} appBar={MyAppBar} />;
export default MyLayout;