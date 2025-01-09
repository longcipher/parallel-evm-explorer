// 封装 fetch 请求方法
const fetchRequest = async (url, options = {}) => {
  const { method = 'GET', params = {}, headers = {}, ...restOptions } = options;
  const config = {
    method,
    headers: { 
      'Content-Type': 'application/json',
      ...(options.token ? { 'Authorization': `Bearer ${options.token}` } : {}),
      ...headers 
    },
    ...restOptions,
  };

  // 处理 GET 请求的 URL 拼接
  if (method === 'GET' && Object.keys(params).length > 0) {
    const queryString = new URLSearchParams(params).toString();
    url = `${url}?${queryString}`;
  }

  // 处理 POST 请求的 body
  if (method === 'POST' && Object.keys(params).length > 0) {
    config.body = JSON.stringify(params);
  }

  try {
    const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL}${url}`, config);

    // 如果响应不是 2xx，则抛出错误
    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(errorData.message || `HTTP error! Status: ${response.status}`);
    }

    // 返回解析后的 JSON 数据
    return await response.json();
  } catch (error) {
    // 错误处理
    console.error('API request error:', error);
    throw error; // 将错误抛出给调用者
  }
};

// 需要封装的常见请求方法
export const get = (url, params = {}, options = {}) => fetchRequest(url, { method: 'GET', params, ...options });
export const post = (url, params = {}, options = {}) => fetchRequest(url, { method: 'POST', params, ...options });

