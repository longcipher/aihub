Feature: Gateway with liter-llm Provider Integration
  As a developer using the aihub gateway
  I want to send LLM requests through an OpenAI-compatible API
  So that I can use any supported provider without changing my client code

  Background:
    Given a running gateway with the following config:
      | provider  | type      | model                        |
      | openai    | openai    | gpt-4o                       |
      | anthropic | anthropic | claude-sonnet-4-20250514      |

  Scenario: Chat completion with OpenAI model
    Given a valid API key for "openai"
    When I send a POST to "/v1/chat/completions" with:
      | model    | messages                          |
      | gpt-4o   | [{"role":"user","content":"Hi"}]  |
    Then the response status is 200
    And the response contains a valid chat completion
    And the "x-genai-provider-name" header is "openai"

  Scenario: Chat completion with Anthropic model
    Given a valid API key for "anthropic"
    When I send a POST to "/v1/chat/completions" with:
      | model                        | messages                          |
      | claude-sonnet-4-20250514     | [{"role":"user","content":"Hi"}]  |
    Then the response status is 200
    And the response contains a valid chat completion
    And the "x-genai-provider-name" header is "anthropic"

  Scenario: Streaming chat completion
    Given a valid API key for "openai"
    When I send a streaming POST to "/v1/chat/completions" with:
      | model  | stream |
      | gpt-4o | true   |
    Then the response is an SSE stream
    And each chunk has the correct OpenAI streaming format

  Scenario: Embeddings request
    Given a valid API key for "openai"
    When I send a POST to "/v1/embeddings" with:
      | model              | input    |
      | text-embedding-3-small | hello  |
    Then the response status is 200
    And the response contains embedding data

  Scenario: Model not found returns 404
    When I send a POST to "/v1/chat/completions" with:
      | model         | messages                          |
      | nonexistent   | [{"role":"user","content":"Hi"}]  |
    Then the response status is 404

  Scenario: Health endpoint
    When I send a GET to "/health"
    Then the response status is 200
    And the response body is "Working!"

  Scenario: Models list endpoint
    When I send a GET to "/v1/models"
    Then the response status is 200
    And the response contains a list of available models
