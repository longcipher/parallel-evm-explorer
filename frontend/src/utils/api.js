const fetchRequest = async (url, options = {}) => {
  const { method = "GET", params = {}, headers = {}, ...restOptions } = options;
  const config = {
    method,
    headers: {
      "Content-Type": "application/json",
      ...(options.token ? { Authorization: `Bearer ${options.token}` } : {}),
      ...headers,
    },
    ...restOptions,
  };

  if (method === "GET" && Object.keys(params).length > 0) {
    const queryString = new URLSearchParams(params).toString();
    url = `${url}?${queryString}`;
  }

  if (method === "POST" && Object.keys(params).length > 0) {
    config.body = JSON.stringify(params);
  }

  try {
    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_SERVER}${url}`,
      config,
    );

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(
        errorData.message || `HTTP error! Status: ${response.status}`,
      );
    }

    return await response.json();
  } catch (error) {
    console.error("API request error:", error);
    throw error;
  }
};

export const get = (url, params = {}, options = {}) =>
  fetchRequest(url, { method: "GET", params, ...options });
export const post = (url, params = {}, options = {}) =>
  fetchRequest(url, { method: "POST", params, ...options });
