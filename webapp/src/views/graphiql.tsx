
import { useEffect } from "react";
import { useSafeSetState } from "react-admin";
import { Box } from "@mui/material";
import { GraphiQL } from "graphiql";
import { getRawAuthorization } from "../components/auth_provider";
import type { GraphQLSchema } from "graphql";
import { envs } from "../components/cypher";
import Loading from "./loading";
import 'graphiql/graphiql.css';


const Graphiql = () => {
  const [schema, setSchema] = useSafeSetState<GraphQLSchema>();
  const origin = envs[localStorage.getItem("environment")].url;

  const fetcher = async (graphQLParams, opts) => {
    const graphqlUrl = `${origin}/graphql`;
    const body = JSON.stringify(graphQLParams);
    const { headers = {} } = opts;
    headers["Authentication"] = await getRawAuthorization(graphqlUrl, "POST", body);

    const response = await fetch(graphqlUrl, {
      method: 'post',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        ...headers,
      },
      body: body,
    });

    return response.json();
  }

  useEffect(() => {
    const init = async () => {
      const response = await fetch(`${origin}/graphql/introspect`);
      setSchema(await response.json());
    }
    init();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (!schema) return <Loading />;

  return(
    <Box sx={{height: "100vh"}}>
      <GraphiQL
        fetcher={fetcher}
        schema={schema}
      />
    </Box>
  )
}

export default Graphiql;
