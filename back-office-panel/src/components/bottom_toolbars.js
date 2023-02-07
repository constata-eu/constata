import {Toolbar, SaveButton} from 'react-admin';

const OnlySaveToolbar = ({alwaysEnable, label}) => (
  <Toolbar>
    <SaveButton alwaysEnable={alwaysEnable} label={label} />
  </Toolbar>
);

export {OnlySaveToolbar};