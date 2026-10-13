"use client";

import {
  type AxiosError,
  HttpStatusCode,
  type InternalAxiosRequestConfig,
} from "axios";
import { API_HTTP_URL } from "configs/constants";
import { RootPath } from "configs/routes/constants";
import { client } from "generated/client/client.gen";
import { isSSR } from "lib/utils";
import Router from "next/router";

if (!isSSR()) {
  client.setConfig({
    baseURL: API_HTTP_URL,
    withCredentials: true,
  });

  client.instance.interceptors.request.use(
    async (requestConfig: InternalAxiosRequestConfig) => requestConfig,
  );

  client.instance.interceptors.response.use(
    (response) => response,
    async (error: AxiosError) => {
      if (error.response) {
        switch (error.response.status as HttpStatusCode) {
          case HttpStatusCode.Unauthorized:
            await Router.push(
              `/${RootPath.SignIn}`,
              `/${RootPath.SignIn}${window.location.search}`,
            );
            break;
          default:
            return Promise.reject(error);
        }
        throw error;
      } else {
        return Promise.reject(error);
      }
    },
  );
}
